// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Russian (`ru`).
class AppLocalizationsRu extends AppLocalizations {
  AppLocalizationsRu([String locale = 'ru']) : super(locale);

  @override
  String get appTitle => 'Dicta';

  @override
  String get dashboardTitle => 'Дашборд';

  @override
  String get historyTitle => 'История';

  @override
  String get modelsTitle => 'Модели';

  @override
  String get settingsTitle => 'Настройки';

  @override
  String get daemonUnavailable => 'демон недоступен';

  @override
  String get placeholderScreen => 'раздел появится в следующих фазах';

  @override
  String get daemonStateIdle => 'ожидание';

  @override
  String get daemonStateRecording => 'запись';

  @override
  String get daemonStateTranscribing => 'распознавание';

  @override
  String stateLabel(String state) {
    return 'состояние: $state';
  }

  @override
  String levelSeconds(String level, String seconds) {
    return 'уровень $level · $seconds с';
  }

  @override
  String dashRecordingTime(String time) {
    return 'время $time';
  }

  @override
  String get dashStartButton => 'начать запись';

  @override
  String get dashStopButton => 'остановить';

  @override
  String get dashCancelButton => 'отмена';

  @override
  String get dashTranscribing => 'распознаю…';

  @override
  String get dashLastTextTitle => 'последний текст';

  @override
  String get dashNoText => 'нет распознанного текста';

  @override
  String get dashNoTextHint =>
      'текст появится после остановки записи из приложения';

  @override
  String get dashCopyButton => 'скопировать';

  @override
  String get dashCopied => 'скопировано';

  @override
  String get dashInjectButton => 'вставить заново';

  @override
  String get dashInjectHint => 'вставка идёт в активное окно';

  @override
  String get dashInjectFailed => 'ошибка вставки';
}
