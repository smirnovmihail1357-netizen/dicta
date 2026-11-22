// The single fake WindowShell seam. Records close-to-tray bookkeeping without
// touching window_manager (no platform channels in the test runner).
import 'package:dicta_app/core/window_shell.dart';

class FakeWindowShell implements WindowShell {
  int preventCloseCount = 0;
  int hideCount = 0;
  int showCount = 0;
  int destroyCount = 0;
  bool visible = true;

  void Function()? onCloseCallback;

  @override
  Future<void> preventClose() async {
    preventCloseCount++;
  }

  @override
  void onCloseRequest(void Function() cb) => onCloseCallback = cb;

  @override
  Future<void> hide() async {
    hideCount++;
    visible = false;
  }

  @override
  Future<void> show() async {
    showCount++;
    visible = true;
  }

  @override
  Future<bool> isVisible() async => visible;

  @override
  Future<void> destroy() async {
    destroyCount++;
  }

  /// Simulates the OS close button (the veto is configured by preventClose).
  void clickCloseButton() => onCloseCallback?.call();
}
