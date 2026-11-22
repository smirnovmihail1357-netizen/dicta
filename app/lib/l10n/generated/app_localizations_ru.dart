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

  @override
  String get dashCommandFailed => 'ошибка команды';

  @override
  String get dashCommandFailedBody => 'команда не выполнена';

  @override
  String get dashInjected => 'текст вставлен';

  @override
  String get calibrationTitle => 'порог тишины';

  @override
  String get calibrationEmptyHint => 'начните запись, чтобы увидеть уровень';

  @override
  String get calibrationApplyHint => 'отпусти ползунок — порог применится';

  @override
  String get calibrationAutoStopHint =>
      'включён авто-стоп — для калибровки нажимай отмена';

  @override
  String calibrationThreshold(String value) {
    return 'порог $value';
  }

  @override
  String get calibrationHistogramLabel => 'гистограмма уровня звука';

  @override
  String get diagTitle => 'диагностика';

  @override
  String get diagDaemonAlive => 'демон жив';

  @override
  String get diagSocketAnswers => 'сокет отвечает';

  @override
  String get diagStatusNote => 'обе строки — один и тот же тест сокета';

  @override
  String get diagModelReady => 'модель готова';

  @override
  String get diagModelLoading => 'модель загружается…';

  @override
  String get diagModelUnavailable => 'модель: данных нет';

  @override
  String get diagModelFetching => 'данные модели загружаются…';

  @override
  String get diagModelFetchFailed => 'модель: не удалось получить данные';

  @override
  String get diagModelNote => 'по данным демона — гарантию даёт живая диктовка';

  @override
  String get diagHotkeyManual => 'хоткей — проверь вручную: dictactl toggle';

  @override
  String get diagInjectManual => 'вставку проверь живой диктовкой';

  @override
  String get diagRecheck => 'перепроверить';

  @override
  String get diagStartHint => 'запусти: systemctl --user start dictad';
}
