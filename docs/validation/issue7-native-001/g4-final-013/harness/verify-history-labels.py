"""Independent actual HistoryModel regressions; all rows are synthetic."""
import unittest
from PySide6.QtCore import Qt
from PySide6.QtWidgets import QApplication
from drive_native_gui.history import HistorySource,HistoryModel

APP=QApplication.instance() or QApplication([])
SOURCE=HistorySource('test','测试挂载',None,'test-task','/isolated')
class PermissionHistoryLabels(unittest.TestCase):
    def model(self,action):
        model=HistoryModel();row=dict(id=1,seq=1,completed_at=1,name='合成对象',path_at_event='A/合成对象',action=action,
            result='pending',attempts=1,error_summary='服务器返回权限相关响应，提交状态待核对',
            detail={'permission_response':{'action':'upload','code':0x9004}})
        model.replace([(SOURCE,row)]);return model,row
    def test_mkdir_label_is_chinese(self):
        model,_=self.model('mkdir');self.assertEqual(model.data(model.index(0,3)),'创建文件夹')
    def test_replace_label_is_chinese(self):
        model,_=self.model('replace');self.assertEqual(model.data(model.index(0,3)),'替换文件')
    def test_confirmed_row_removes_hint_from_tooltip_and_retains_response_detail(self):
        model,row=self.model('replace');self.assertIn('提交状态待核对',model.data(model.index(0,4),Qt.ToolTipRole))
        final=dict(row,result='ok',error_summary=None,error_kind=None);model.replace([(SOURCE,final)])
        self.assertEqual(model.data(model.index(0,4)),'已完成')
        self.assertNotIn('提交状态待核对',model.data(model.index(0,4),Qt.ToolTipRole))
        self.assertEqual(model.data(model.index(0,4),Qt.UserRole)[1]['detail'],row['detail'])
if __name__=='__main__':unittest.main()
