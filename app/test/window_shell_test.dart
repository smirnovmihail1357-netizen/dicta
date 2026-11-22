// Close-to-tray seam in isolation (no ProviderContainer, no plugin). The close
// button only ever reaches hide(); destroy is a separate explicit call, which
// TrayController wires exclusively to the Exit menu item.
import 'package:flutter_test/flutter_test.dart';

import 'fake_window_shell.dart';

void main() {
  test('close request hides without destroying (TRAY-02)', () async {
    final shell = FakeWindowShell();
    shell.onCloseRequest(() => shell.hide());

    shell.clickCloseButton();
    await Future<void>.delayed(const Duration(milliseconds: 1));

    expect(shell.hideCount, 1);
    expect(shell.destroyCount, 0);
    expect(shell.visible, isFalse);
  });

  test('destroy happens only through an explicit exit call', () async {
    final shell = FakeWindowShell();
    shell.onCloseRequest(() => shell.hide());

    // No registered close callback means a stray event is inert.
    final inert = FakeWindowShell();
    inert.clickCloseButton();
    expect(inert.destroyCount, 0);

    await shell.destroy();
    expect(shell.destroyCount, 1);
    expect(shell.hideCount, 0);
  });
}
