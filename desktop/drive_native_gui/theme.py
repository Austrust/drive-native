"""Consistent Qt foreground/background pairs across third-party Mint themes."""
from PySide6.QtCore import Qt
from PySide6.QtGui import QColor, QPalette

def palette_for(dark):
    p = QPalette()
    values = {
        QPalette.Window: "#20242b" if dark else "#f5f6f8",
        QPalette.WindowText: "#f1f3f5" if dark else "#202631",
        QPalette.Base: "#292e37" if dark else "#ffffff",
        QPalette.AlternateBase: "#323945" if dark else "#edf0f5",
        QPalette.Text: "#f1f3f5" if dark else "#202631",
        QPalette.Button: "#292e37" if dark else "#ffffff",
        QPalette.ButtonText: "#f1f3f5" if dark else "#202631",
        QPalette.Highlight: "#355bb0",
        QPalette.HighlightedText: "#ffffff",
        QPalette.ToolTipBase: "#292e37" if dark else "#ffffff",
        QPalette.ToolTipText: "#f1f3f5" if dark else "#202631",
        QPalette.PlaceholderText: "#b5bfce" if dark else "#596575",
        QPalette.Mid: "#aab5c5" if dark else "#626d7c",
        QPalette.Midlight: "#566171" if dark else "#c5ccd6",
        QPalette.Link: "#9ebdff" if dark else "#2855ad",
    }
    for group in (QPalette.Active, QPalette.Inactive, QPalette.Disabled):
        for role, value in values.items(): p.setColor(group, role, QColor(value))
    for role in (QPalette.Text, QPalette.WindowText, QPalette.ButtonText):
        p.setColor(QPalette.Disabled, role, QColor("#aab5c5" if dark else "#626d7c"))
    return p

def install_theme(app):
    hints = app.styleHints()
    scheme = hints.colorScheme()
    dark = scheme == Qt.ColorScheme.Dark or (scheme == Qt.ColorScheme.Unknown and app.palette().color(QPalette.Window).lightness() < 128)
    # Native GTK/Kvantum styles may paint independently of the requested palette.
    app.setStyle("Fusion")
    app.setPalette(palette_for(dark))
    def changed(scheme):
        app.setPalette(palette_for(scheme == Qt.ColorScheme.Dark))
        for widget in app.topLevelWidgets():
            if hasattr(widget, "apply_theme"): widget.apply_theme()
    hints.colorSchemeChanged.connect(changed)


def apply_window_theme(window):
    muted = "#a8b2c2" if window.palette().color(QPalette.Window).lightness() < 128 else "#626d7c"
    stylesheet = """QWidget{font-size:14px;} QMainWindow{background:palette(window);}
        QFrame#sidebar{background:palette(base);border-right:1px solid palette(midlight);}
        QLabel#brand{font-size:22px;font-weight:700;} QLabel#heading{font-size:27px;font-weight:650;}
        QLabel#metric{font-size:24px;font-weight:600;} QLabel#muted{color:MUTED_COLOR;font-size:12px;}
        QFrame#card{background:palette(base);border:1px solid palette(midlight);border-radius:12px;}
        QPushButton{color:palette(button-text);padding:9px 14px;border:1px solid palette(midlight);border-radius:7px;background:palette(base);}
        QPushButton:hover{background:palette(alternate-base);} QPushButton:disabled{color:palette(mid);}
        QPushButton#primary,QPushButton#navigation{background:#446fd1;color:white;border:none;}
        QLineEdit{color:palette(text);selection-background-color:palette(highlight);selection-color:palette(highlighted-text);padding:9px;border:1px solid palette(midlight);border-radius:6px;background:palette(base);}
        QTreeWidget{border:1px solid palette(midlight);border-radius:9px;background:palette(base);color:palette(text);selection-background-color:palette(highlight);selection-color:palette(highlighted-text);outline:none;}
        QTreeWidget::item{padding:10px 6px;} QHeaderView::section{padding:10px;background:palette(alternate-base);border:none;text-align:left;}
        QLabel#message{padding:12px;border-radius:8px;background:palette(alternate-base);}
    """
    window.setStyleSheet(stylesheet.replace("MUTED_COLOR", muted))
