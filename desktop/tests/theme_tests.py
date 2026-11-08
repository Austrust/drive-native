import unittest
from PySide6.QtGui import QPalette
from drive_native_gui.theme import (
    palette_for, STATUS_KINDS, STATUS_COLORS, STATUS_ICONS,
    status_color, status_icon, status_kind,
)

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

    def test_status_kinds_have_distinct_dual_scheme_colors(self):
        for kind in STATUS_KINDS:
            self.assertIn(kind, STATUS_COLORS["light"])
            self.assertIn(kind, STATUS_COLORS["dark"])
            self.assertNotEqual(status_color(kind, dark=False), status_color(kind, dark=True))
            self.assertIn(status_icon(kind), tuple(STATUS_ICONS.values()))
        self.assertEqual(len({STATUS_COLORS["light"][k] for k in STATUS_KINDS}), len(STATUS_KINDS))
        self.assertEqual(len({STATUS_COLORS["dark"][k] for k in STATUS_KINDS}), len(STATUS_KINDS))

    def test_status_color_uses_requested_scheme(self):
        self.assertEqual(status_color("attention", dark=True), STATUS_COLORS["dark"]["attention"])
        self.assertEqual(status_color("attention", dark=False), STATUS_COLORS["light"]["attention"])
        self.assertEqual(status_color("no-such-kind", dark=False), STATUS_COLORS["light"]["unknown"])

    def test_status_kind_maps_connection_and_file_values(self):
        self.assertEqual(status_kind("online"), "idle")
        self.assertEqual(status_kind("needs_attention"), "attention")
        self.assertEqual(status_kind("paused"), "paused")
        self.assertEqual(status_kind("local_complete"), "idle")
        self.assertEqual(status_kind("pending_upload"), "syncing")
        self.assertEqual(status_kind("online_only"), "unknown")
        self.assertEqual(status_kind("conflict"), "attention")
        for source in (None, "", "not-a-status", "removed"):
            self.assertEqual(status_kind(source), "unknown")

if __name__=='__main__': unittest.main()
