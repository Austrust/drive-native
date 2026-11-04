"""Actual frozen ClientBus/Desktop IPC + HistoryPage load; isolated synthetic state."""
import contextlib
import hashlib
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import sys
import time
import traceback

from PySide6.QtCore import QEventLoop, QTimer
from PySide6.QtWidgets import QApplication
from drive_native_gui.bus import ClientBus
from drive_native_gui.history import HistoryPage, HistorySource

root, binary = map(Path, sys.argv[1:3])
assert not (root / 'results.json').exists()
app = QApplication([])
checks = []
beats = []
heartbeat = QTimer()
heartbeat.setInterval(5)
heartbeat.timeout.connect(lambda: beats.append(time.perf_counter()))
heartbeat.start()

def wait_until(predicate, timeout=30):
    loop = QEventLoop()
    timer = QTimer(); timer.setInterval(2)
    deadline = time.monotonic() + timeout
    timer.timeout.connect(lambda: loop.quit() if predicate() or time.monotonic() > deadline else None)
    timer.start(); loop.exec(); timer.stop()
    assert predicate(), 'independent harness wait timed out'

def metrics(start, beat_start):
    times = [start] + beats[beat_start:] + [time.perf_counter()]
    gaps=sorted((b-a)*1000 for a,b in zip(times,times[1:]))
    return {'wall_seconds':times[-1]-start, 'qt_heartbeat_count':len(times)-2,
            'qt_max_heartbeat_gap_ms':max(gaps),
            'qt_p95_heartbeat_gap_ms':gaps[min(len(gaps)-1,int(len(gaps)*.95))],
            'qt_p99_heartbeat_gap_ms':gaps[min(len(gaps)-1,int(len(gaps)*.99))],
            'heartbeat_interval_ms':5, 'functional_gap_budget_ms':250,
            'budget_source':'independent harness functional criterion, not a PLAN threshold'}

def call(bus, query):
    replies=[]; started=time.perf_counter()
    bus.get_sync_history(query, lambda data,error:replies.append((data,error)))
    wait_until(lambda:bool(replies))
    data,error=replies[0]
    assert not error, error
    return data,(time.perf_counter()-started)*1000

def paginate(bus, query):
    result=[]; latencies=[]; cursors=[]
    while True:
        data,ms=call(bus,query)
        rows=data['records']; result.extend(rows); latencies.append(ms)
        assert len(rows)<=query.get('limit',100)
        cursor=data.get('next_cursor')
        if not cursor:break
        assert cursor not in cursors and cursor == str(rows[-1]['seq'])
        cursors.append(cursor); query=dict(query,cursor=cursor)
    return result,latencies

server=None
summary={'scope':'G4 independent synthetic functional evidence, not G5 benchmark',
         'seed_scope':'direct synthetic SQL exercises default retention and real read/IPC/GUI paths; workflow writers covered separately',
         'retention':'default 10000 records; not the maximum configurable retention',
         'timeout_peer':'independent 60-second delayed same-interface peer; actual ClientBus timeout',
         'package_reinstall_history':'未实测', 'checks':checks,
         'server_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}
try:
    state=root/'state'; state.mkdir(mode=0o700)
    subprocess.run([str(binary),'init',str(state)],check=True)
    now=int(time.time())
    with contextlib.closing(sqlite3.connect(state/'state.sqlite')) as db:
        db.executemany('INSERT INTO sync_history(seq,mount,node,name,path_at_event,action,result,attempts,completed_at,operation_key) VALUES(?,?,?,?,?,?,?,?,?,?)',
            [(i,'rootA' if i%2==0 else 'rootB',None,
              f"{'Needle%_' if i%10==0 else 'file'}{i:05d}",f'/synthetic/{i:05d}',
              'upload','failed' if i%4==0 else 'ok',1,now,f'synthetic:{i}') for i in range(1,10002)])
        db.execute("UPDATE metadata SET value='10001' WHERE key='sync_history_sequence'")
        db.commit()
        retention=db.execute('SELECT COUNT(*),MIN(seq),MAX(seq) FROM sync_history').fetchone()
        assert retention==(10000,2,10001),retention
        assert db.execute('PRAGMA integrity_check').fetchone()[0]=='ok'
    checks.append({'name':'default_retention_10001_to_10000','passed':True,'count_min_max':retention})
    serverlog=(root/'server.log').open('w')
    server=subprocess.Popen([str(binary),'serve',str(state)],stdout=serverlog,stderr=subprocess.STDOUT)
    actual=ClientBus(instance='0'*31+'2'); delayed=ClientBus(instance='0'*31+'1')
    wait_until(lambda:bool(actual.owner and delayed.owner),30)

    start=time.perf_counter(); b=len(beats)
    rows,ms=paginate(actual,{'limit':500})
    assert [r['seq'] for r in rows]==list(range(10001,1,-1))
    assert len({r['id'] for r in rows})==10000
    timing=metrics(start,b)
    assert timing['qt_max_heartbeat_gap_ms']<250,timing
    checks.append(dict(name='actual_ipc_10000_pagination',passed=True,pages=len(ms),records=len(rows),
                       request_ms=ms,**timing))

    rows,ms=paginate(actual,{'limit':100,'mount':'rootA','result':'failed','name':'nEEdLe%_'})
    assert [r['seq'] for r in rows]==list(range(10000,1,-20))
    checks.append(dict(name='actual_ipc_combined_mount_result_literal_name_filter',passed=True,
                       pages=len(ms),records=len(rows),request_ms=ms))

    source=HistorySource('actual','Synthetic history',actual,'synthetic-task','root')
    page=HistoryPage(lambda:{'actual':source}); page.resize(1000,700)
    start=time.perf_counter(); b=len(beats); page.show()
    wait_until(lambda:not page.pending and bool(page.started))
    pages=1
    while page.cursors.get('actual'):
        page.load_more(); wait_until(lambda:not page.pending); pages+=1
    assert not page.errors and len(page.model.records)==10000
    assert not page.more.isEnabled()
    assert [r['seq'] for _,r in page.model.records]==list(range(10001,1,-1))
    timing=metrics(start,b); assert timing['qt_max_heartbeat_gap_ms']<250,timing
    page.grab().save(str(root/'history-10000.png'))
    checks.append(dict(name='actual_history_page_100_pages_gui_responsive',passed=True,pages=pages,
                       records=page.model.rowCount(),**timing))
    page.name_filter.setText('nEEdLe%_'); page.result_filter.setCurrentIndex(page.result_filter.findData('failed'))
    page.debounce.stop(); page.refresh()
    wait_until(lambda:not page.pending and bool(page.started))
    while page.cursors.get('actual'):
        page.load_more(); wait_until(lambda:not page.pending)
    assert [r['seq'] for _,r in page.model.records]==list(range(10000,1,-20))
    checks.append(dict(name='actual_history_page_result_name_filter',passed=True,records=page.model.rowCount()))
    page.hide()

    slow_source=HistorySource('slow','Delayed synthetic peer',delayed,'slow-task','root')
    slow_page=HistoryPage(lambda:{'slow':slow_source}); slow_page.resize(1000,700)
    replies=[]; healthy=[]; start=time.perf_counter(); b=len(beats)
    delayed.get_sync_history({'limit':100},lambda data,error:replies.append((time.perf_counter(),data,error)))
    slow_page.show()
    QTimer.singleShot(250,lambda:actual.get_sync_history({'limit':1},lambda data,error:healthy.append((time.perf_counter(),data,error))))
    wait_until(lambda:bool(replies) and not slow_page.pending,22)
    elapsed=replies[0][0]-start; error=replies[0][2]
    assert error and 'NoReply' in error,(elapsed,error)
    assert replies[0][1] is None and len(replies)==1 and not delayed.pending
    assert 14<=elapsed<=18,elapsed
    assert slow_page.errors and not slow_page.pending and slow_page.refresh_button.isEnabled()
    assert healthy and not healthy[0][2] and len(healthy[0][1]['records'])==1
    assert healthy[0][0]-start<2,healthy[0][0]-start
    timing=metrics(start,b); assert timing['qt_heartbeat_count']>1000 and timing['qt_max_heartbeat_gap_ms']<250,timing
    slow_page.grab().save(str(root/'history-timeout.png'))
    checks.append(dict(name='real_clientbus_15s_timeout_page_error_healthy_peer_and_heartbeat',passed=True,
                       timeout_seconds=elapsed,error=error,healthy_peer_reply_seconds=healthy[0][0]-start,
                       page_error=dict(slow_page.errors),**timing))
    slow_page.hide()
except BaseException:
    summary['failure']=traceback.format_exc()
    traceback.print_exc()
finally:
    if server:
        server.send_signal(signal.SIGINT)
        try:server.wait(timeout=10)
        except subprocess.TimeoutExpired:server.kill();server.wait()
        summary['server_exit_code']=server.returncode
        serverlog.close()
    summary['passed']=len(checks)==6 and not summary.get('failure') and summary.get('server_exit_code')==0
    summary['passed_count']=sum(c['passed'] for c in checks)
    summary['failed_count']=int(not summary['passed'])
    summary['ignored_count']=0
    summary['harness_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    (root/'results.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps(summary,ensure_ascii=False),flush=True)
sys.exit(0 if summary['passed'] else 1)
