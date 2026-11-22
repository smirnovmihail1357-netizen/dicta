/// The app-window seam. The legacy `window_manager` calls live only in this
/// file; the bootstrap uses the abstract interface so tests can inject
/// FakeWindowShell (mirror of core/tray.dart and core/hf/hf_client.dart).
library;

import 'package:window_manager/legacy.dart';

abstract interface class WindowShell {
  /// Veto the OS close request: the X button hides to tray instead.
  Future<void> preventClose();

  /// Registers the callback fired by the native close button.
  void onCloseRequest(void Function() cb);

  Future<void> hide();
  Future<void> show();
  Future<bool> isVisible();

  /// Force-close, bypassing the veto. Only the tray's Exit item may call this.
  Future<void> destroy();
}

class WindowManagerShell implements WindowShell {
  WindowListener? _listener;

  @override
  Future<void> preventClose() => windowManager.setPreventClose(true);

  @override
  void onCloseRequest(void Function() cb) {
    // One listener for the process lifetime; repeated registration would stack
    // duplicate hide() calls on every close request.
    if (_listener != null) return;
    final listener = _CloseListener(cb);
    _listener = listener;
    windowManager.addListener(listener);
  }

  @override
  Future<void> hide() => windowManager.hide();

  @override
  Future<void> show() => windowManager.show();

  @override
  Future<bool> isVisible() => windowManager.isVisible();

  @override
  Future<void> destroy() => windowManager.destroy();
}

class _CloseListener extends WindowListener {
  _CloseListener(this._onClose);

  final void Function() _onClose;

  @override
  void onWindowClose() => _onClose();
}
