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
  String get daemonStateIdle => 'ожидание';

  @override
  String get daemonStateRecording => 'запись';

  @override
  String get daemonStateTranscribing => 'распознавание';

  @override
  String get dashStatusCardTitle => 'состояние';

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
  String get histEmpty => 'журнал пуст';

  @override
  String get histEmptyHint =>
      'записи появляются после диктовок из приложения — глобальный хоткей демона приложению не виден';

  @override
  String get histSearchHint => 'поиск по тексту';

  @override
  String get histSearchNoMatches => 'ничего не найдено';

  @override
  String get histDeleteButton => 'удалить';

  @override
  String get histDeleted => 'удалено';

  @override
  String get histUndo => 'вернуть';

  @override
  String get histSearchSemantics => 'поиск по журналу';

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
  String get diagModelUnavailable => 'данных о модели нет';

  @override
  String get diagModelFetching => 'данные модели загружаются…';

  @override
  String get diagModelFetchFailed => 'не удалось получить данные о модели';

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

  @override
  String get confGroupInsert => 'вставка';

  @override
  String get confPasteDelayLabel => 'задержка перед вставкой, мс';

  @override
  String get confGroupRecord => 'запись';

  @override
  String get confGroupEngine => 'движок';

  @override
  String get confGroupSystem => 'система';

  @override
  String get confMinLevelLabel => 'порог';

  @override
  String get confAutoStopLabel => 'авто-стоп';

  @override
  String get confAutoStopLevelLabel => 'порог авто-стопа';

  @override
  String get confAutoStopAfterLabel => 'пауза перед стопом, мс';

  @override
  String get confAutoStopGraceLabel => 'задержка авто-стопа, мс';

  @override
  String get confMaxRecordLabel => 'максимум записи, мс';

  @override
  String get confInjectModeLabel => 'способ вставки';

  @override
  String get confPasteKeysLabel => 'горячие клавиши вставки';

  @override
  String get confPasteKeysHint => 'ctrl+v';

  @override
  String get confClipboardRestoreLabel => 'возврат буфера, мс';

  @override
  String get confAtspiLabel => 'atspi: любой роли';

  @override
  String get confModelLabel => 'модель';

  @override
  String get confUseGpuLabel => 'gpu';

  @override
  String get confGpuDeviceLabel => 'устройство gpu';

  @override
  String get confThreadsLabel => 'потоки';

  @override
  String get confBeamLabel => 'размер луча';

  @override
  String get confLanguageLabel => 'язык';

  @override
  String get confConfigPathLabel => 'путь конфигурации';

  @override
  String get confEngineHint => 'эти настройки перезагружают модель (~секунды)';

  @override
  String get confEngineConfirmTitle => 'перезагрузить модель?';

  @override
  String get confEngineConfirmBody =>
      'изменение перезагрузит модель — на секунды диктовка будет недоступна';

  @override
  String get confConfirm => 'продолжить';

  @override
  String get confDecline => 'отмена';

  @override
  String get confSeconds => 'с';

  @override
  String get confMilliseconds => 'мс';

  @override
  String get modelInstalledTitle => 'установленные';

  @override
  String get modelActive => 'активна';

  @override
  String get modelSelectButton => 'выбрать';

  @override
  String get modelDeleteButton => 'удалить';

  @override
  String get modelDeleteConfirmTitle => 'удалить модель?';

  @override
  String get modelDeleteConfirmBody =>
      'файл будет удалён безвозвратно — вернуть нельзя';

  @override
  String get modelDeleteConfirm => 'удалить';

  @override
  String get modelDecline => 'отмена';

  @override
  String get modelDeleted => 'удалено';

  @override
  String get modelEmpty => 'моделей нет — скачай из каталога ниже';

  @override
  String get modelFetchFailed => 'не удалось получить список моделей';

  @override
  String get modelFetchRetry => 'повторить';

  @override
  String get modelDeleteBlocked => 'модель активна или вне каталога';

  @override
  String get modelCatalogTitle => 'каталог проверенных';

  @override
  String get modelNoteTiny => 'сверхбыстрая, для теста';

  @override
  String get modelNoteBase => 'быстрая, простые фразы';

  @override
  String get modelNoteSmall => 'баланс скорости и качества';

  @override
  String get modelNoteMedium => 'хорошее качество';

  @override
  String get modelNoteLargeTurbo => 'лучшее качество при высокой скорости';

  @override
  String get modelNoteLarge => 'максимум качества, медленнее';

  @override
  String get modelVramLabel => 'лимит vram, гб';

  @override
  String get modelVramOver => 'больше лимита vram';

  @override
  String get modelAlreadyInstalled => 'уже есть';

  @override
  String get modelSearchHint => 'поиск на hugging face';

  @override
  String get modelSearchSemantics => 'поиск моделей на hugging face';

  @override
  String get modelSearchButton => 'найти';

  @override
  String get modelShowMore => 'показать ещё';

  @override
  String get modelSortDownloads => 'загрузки';

  @override
  String get modelSortLikes => 'лайки';

  @override
  String get modelSortModified => 'обновлённые';

  @override
  String get modelAsrFilter => 'только распознавание речи';

  @override
  String get modelResultsEmpty => 'ничего не нашлось';

  @override
  String get modelTreeEmpty => 'файлов .bin нет';

  @override
  String get modelSubdirBack => 'назад';

  @override
  String get modelRepoClosed => 'репозиторий закрыт или не найден';

  @override
  String get modelDirectLinkLabel => 'прямая ссылка на .bin';

  @override
  String get modelDirectLinkHint => 'https://…/ggml-….bin';

  @override
  String get modelDownloadButton => 'скачать';

  @override
  String get modelCancelDownload => 'отменить загрузку';

  @override
  String get modelDownloadDone => 'модель скачана — выбери её в списке';

  @override
  String get modelDownloadCancelled => 'загрузка прервана';

  @override
  String get modelDownloadFailed => 'загрузка не удалась';

  @override
  String modelDownloadingMb(String mb) {
    return '$mb МБ';
  }
}
