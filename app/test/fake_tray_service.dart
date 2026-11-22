// The single fake TrayService seam. No plugin channels: tests drive the six
// registered menu callbacks directly and inspect recorded icon transitions.
import 'package:dicta_app/core/tray.dart';

class FakeTrayService implements TrayService {
  FakeTrayService({this.failStart = false});

  /// Simulates an SNI host / panel missing at boot (WR-02).
  final bool failStart;

  final List<TrayIconState> iconCalls = <TrayIconState>[];
  int startCount = 0;
  int disposeCount = 0;

  final Map<String, void Function()> _menu = <String, void Function()>{};

  @override
  Future<void> start() async {
    if (failStart) throw StateError('no SNI host');
    startCount++;
  }

  @override
  Future<void> setIconState(TrayIconState state) async {
    iconCalls.add(state);
  }

  @override
  void onMenuShowHide(void Function() cb) => _menu['showHide'] = cb;

  @override
  void onMenuStart(void Function() cb) => _menu['start'] = cb;

  @override
  void onMenuStop(void Function() cb) => _menu['stop'] = cb;

  @override
  void onMenuCancel(void Function() cb) => _menu['cancel'] = cb;

  @override
  void onMenuSettings(void Function() cb) => _menu['settings'] = cb;

  @override
  void onMenuExit(void Function() cb) => _menu['exit'] = cb;

  @override
  Future<void> dispose() async {
    disposeCount++;
  }

  // Test-side menu activation (a real panel owns the click on Linux, so the
  // fake exposes the registered callbacks rather than any native event).
  void clickShowHide() => _menu['showHide']?.call();
  void clickStart() => _menu['start']?.call();
  void clickStop() => _menu['stop']?.call();
  void clickCancel() => _menu['cancel']?.call();
  void clickSettings() => _menu['settings']?.call();
  void clickExit() => _menu['exit']?.call();

  bool hasCallback(String key) => _menu.containsKey(key);
}
