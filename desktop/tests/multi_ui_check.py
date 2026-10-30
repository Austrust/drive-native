import time,json
from PySide6.QtWidgets import QApplication
from drive_native_gui.manager import MountManager
app=QApplication([]);win=MountManager();win.show()
end=time.monotonic()+8
while time.monotonic()<end:
 app.processEvents()
 if len(win.items)==3 and all(i.text(1) not in ("正在读取状态…","状态未知","未启动") for i in win.items.values()):break
 time.sleep(.02)
assert len(win.items)==3
assert win.items['a'*32].text(1)=='已暂停'
assert win.items['b'*32].text(1)=='等待解锁'
assert win.items['a'*32].text(0)=='科研 文档'
assert win.items['b'*32].text(0)=='团队资料'
print(json.dumps({'manager_three_tasks':True,'state_routing':True}))
win.close()
