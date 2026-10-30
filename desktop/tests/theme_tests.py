import unittest
from PySide6.QtGui import QPalette
from drive_native_gui.theme import palette_for

def luminance(c):
    channels=[v/255 for v in (c.red(),c.green(),c.blue())]
    channels=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in channels]
    return sum(v*w for v,w in zip(channels,(.2126,.7152,.0722)))

class ThemeTests(unittest.TestCase):
    def test_all_text_pairs_have_readable_contrast(self):
        for dark in (False,True):
            p=palette_for(dark)
            for group in (QPalette.Active,QPalette.Inactive,QPalette.Disabled):
                for fg,bg in ((QPalette.WindowText,QPalette.Window),(QPalette.Text,QPalette.Base),(QPalette.Text,QPalette.AlternateBase),(QPalette.ButtonText,QPalette.Button),(QPalette.HighlightedText,QPalette.Highlight),(QPalette.PlaceholderText,QPalette.Base),(QPalette.ToolTipText,QPalette.ToolTipBase)):
                    a,b=sorted((luminance(p.color(group,fg)),luminance(p.color(group,bg))))
                    self.assertGreaterEqual((b+.05)/(a+.05),4.5,(dark,group,fg,bg))
if __name__=='__main__': unittest.main()
