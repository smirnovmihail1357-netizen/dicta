import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_ru.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'generated/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[Locale('ru')];

  /// Window title of the application
  ///
  /// In ru, this message translates to:
  /// **'Dicta'**
  String get appTitle;

  /// Dashboard destination in the sidebar
  ///
  /// In ru, this message translates to:
  /// **'Дашборд'**
  String get dashboardTitle;

  /// History destination in the sidebar
  ///
  /// In ru, this message translates to:
  /// **'История'**
  String get historyTitle;

  /// Models destination in the sidebar
  ///
  /// In ru, this message translates to:
  /// **'Модели'**
  String get modelsTitle;

  /// Settings destination in the sidebar
  ///
  /// In ru, this message translates to:
  /// **'Настройки'**
  String get settingsTitle;

  /// Persistent banner when the dictad socket cannot be reached
  ///
  /// In ru, this message translates to:
  /// **'демон недоступен'**
  String get daemonUnavailable;

  /// Daemon state: idle
  ///
  /// In ru, this message translates to:
  /// **'ожидание'**
  String get daemonStateIdle;

  /// Daemon state: recording
  ///
  /// In ru, this message translates to:
  /// **'запись'**
  String get daemonStateRecording;

  /// Daemon state: transcribing
  ///
  /// In ru, this message translates to:
  /// **'распознавание'**
  String get daemonStateTranscribing;

  /// Noun title of the dashboard status card
  ///
  /// In ru, this message translates to:
  /// **'состояние'**
  String get dashStatusCardTitle;

  /// Live signal level and elapsed seconds on the dashboard
  ///
  /// In ru, this message translates to:
  /// **'уровень {level} · {seconds} с'**
  String levelSeconds(String level, String seconds);

  /// Elapsed recording time derived from the daemon seconds field
  ///
  /// In ru, this message translates to:
  /// **'время {time}'**
  String dashRecordingTime(String time);

  /// Start recording button, active only while idle
  ///
  /// In ru, this message translates to:
  /// **'начать запись'**
  String get dashStartButton;

  /// Stop recording button, active only while recording
  ///
  /// In ru, this message translates to:
  /// **'остановить'**
  String get dashStopButton;

  /// Cancel recording button, deliberately disabled while transcribing
  ///
  /// In ru, this message translates to:
  /// **'отмена'**
  String get dashCancelButton;

  /// Spinner caption tied to the polled transcribing state
  ///
  /// In ru, this message translates to:
  /// **'распознаю…'**
  String get dashTranscribing;

  /// Title of the last recognized text card
  ///
  /// In ru, this message translates to:
  /// **'последний текст'**
  String get dashLastTextTitle;

  /// Honest empty state: hotkey dictations are not visible to the app
  ///
  /// In ru, this message translates to:
  /// **'нет распознанного текста'**
  String get dashNoText;

  /// Explains why the last text card can be empty
  ///
  /// In ru, this message translates to:
  /// **'текст появится после остановки записи из приложения'**
  String get dashNoTextHint;

  /// Copies the last recognized text through the clipboard seam
  ///
  /// In ru, this message translates to:
  /// **'скопировать'**
  String get dashCopyButton;

  /// Local snackbar confirmation after an explicit copy click
  ///
  /// In ru, this message translates to:
  /// **'скопировано'**
  String get dashCopied;

  /// Re-inserts the last recognized text with the daemon inject command
  ///
  /// In ru, this message translates to:
  /// **'вставить заново'**
  String get dashInjectButton;

  /// Honest note about inject focus; cross-window path is the daemon hotkey
  ///
  /// In ru, this message translates to:
  /// **'вставка идёт в активное окно'**
  String get dashInjectHint;

  /// Label above the daemon's verbatim inject error
  ///
  /// In ru, this message translates to:
  /// **'ошибка вставки'**
  String get dashInjectFailed;

  /// Neutral label above a failed start/stop/cancel error; inject keeps its own heading
  ///
  /// In ru, this message translates to:
  /// **'ошибка команды'**
  String get dashCommandFailed;

  /// Body shown when a command failed without a verbatim daemon error
  ///
  /// In ru, this message translates to:
  /// **'команда не выполнена'**
  String get dashCommandFailedBody;

  /// Local snackbar confirmation after a successful re-inject
  ///
  /// In ru, this message translates to:
  /// **'текст вставлен'**
  String get dashInjected;

  /// Title of the empty state on the history screen
  ///
  /// In ru, this message translates to:
  /// **'журнал пуст'**
  String get histEmpty;

  /// Honest empty-state hint: daemon hotkey dictations are invisible to the app
  ///
  /// In ru, this message translates to:
  /// **'записи появляются после диктовок из приложения — глобальный хоткей демона приложению не виден'**
  String get histEmptyHint;

  /// Hint on the history journal search field
  ///
  /// In ru, this message translates to:
  /// **'поиск по тексту'**
  String get histSearchHint;

  /// Shown when the search query matches no journal entry
  ///
  /// In ru, this message translates to:
  /// **'ничего не найдено'**
  String get histSearchNoMatches;

  /// Button that removes one entry from the in-memory journal
  ///
  /// In ru, this message translates to:
  /// **'удалить'**
  String get histDeleteButton;

  /// Local snackbar confirmation after deleting a journal entry
  ///
  /// In ru, this message translates to:
  /// **'удалено'**
  String get histDeleted;

  /// Snackbar action that restores a just-deleted journal entry at its original index
  ///
  /// In ru, this message translates to:
  /// **'вернуть'**
  String get histUndo;

  /// Semantic label of the history journal search field
  ///
  /// In ru, this message translates to:
  /// **'поиск по журналу'**
  String get histSearchSemantics;

  /// Title of the live level histogram and calibration card
  ///
  /// In ru, this message translates to:
  /// **'порог тишины'**
  String get calibrationTitle;

  /// Shown while the level history ring buffer is empty
  ///
  /// In ru, this message translates to:
  /// **'начните запись, чтобы увидеть уровень'**
  String get calibrationEmptyHint;

  /// Explains the one-command-per-gesture calibration contract
  ///
  /// In ru, this message translates to:
  /// **'отпусти ползунок — порог применится'**
  String get calibrationApplyHint;

  /// Warns that silence may stop recording before calibration finishes
  ///
  /// In ru, this message translates to:
  /// **'включён авто-стоп — для калибровки нажимай отмена'**
  String get calibrationAutoStopHint;

  /// Numeric readout of the current silence threshold, always visible next to the histogram
  ///
  /// In ru, this message translates to:
  /// **'порог {value}'**
  String calibrationThreshold(String value);

  /// Static read-only semantics label for the level histogram; the Slider below is the accessible control
  ///
  /// In ru, this message translates to:
  /// **'гистограмма уровня звука'**
  String get calibrationHistogramLabel;

  /// Title of the onboarding diagnostics checklist
  ///
  /// In ru, this message translates to:
  /// **'диагностика'**
  String get diagTitle;

  /// Automatic row from the polled daemon status
  ///
  /// In ru, this message translates to:
  /// **'демон жив'**
  String get diagDaemonAlive;

  /// Automatic row: same transport test as daemon alive, shown honestly
  ///
  /// In ru, this message translates to:
  /// **'сокет отвечает'**
  String get diagSocketAnswers;

  /// Honest note that the two status rows share one signal
  ///
  /// In ru, this message translates to:
  /// **'обе строки — один и тот же тест сокета'**
  String get diagStatusNote;

  /// Green row when settings report a model and no reload
  ///
  /// In ru, this message translates to:
  /// **'модель готова'**
  String get diagModelReady;

  /// Neutral row while the daemon is still loading its model
  ///
  /// In ru, this message translates to:
  /// **'модель загружается…'**
  String get diagModelLoading;

  /// Red row when the daemon is unreachable or reports no model
  ///
  /// In ru, this message translates to:
  /// **'данных о модели нет'**
  String get diagModelUnavailable;

  /// Neutral row while the settings snapshot has not landed yet
  ///
  /// In ru, this message translates to:
  /// **'данные модели загружаются…'**
  String get diagModelFetching;

  /// Distinct row when the settings fetch or parse failed
  ///
  /// In ru, this message translates to:
  /// **'не удалось получить данные о модели'**
  String get diagModelFetchFailed;

  /// Honest caveat for the model readiness proxy
  ///
  /// In ru, this message translates to:
  /// **'по данным демона — гарантию даёт живая диктовка'**
  String get diagModelNote;

  /// Manual hint; the app never triggers the daemon hotkey
  ///
  /// In ru, this message translates to:
  /// **'хоткей — проверь вручную: dictactl toggle'**
  String get diagHotkeyManual;

  /// Manual hint; the app never inserts text from diagnostics
  ///
  /// In ru, this message translates to:
  /// **'вставку проверь живой диктовкой'**
  String get diagInjectManual;

  /// Forces one status poll and one settings refetch
  ///
  /// In ru, this message translates to:
  /// **'перепроверить'**
  String get diagRecheck;

  /// Repair hint shown next to red daemon rows
  ///
  /// In ru, this message translates to:
  /// **'запусти: systemctl --user start dictad'**
  String get diagStartHint;

  /// Settings group title: text insertion
  ///
  /// In ru, this message translates to:
  /// **'вставка'**
  String get confGroupInsert;

  /// Label of the paste_delay_ms setting
  ///
  /// In ru, this message translates to:
  /// **'задержка перед вставкой, мс'**
  String get confPasteDelayLabel;

  /// Settings group title: audio recording
  ///
  /// In ru, this message translates to:
  /// **'запись'**
  String get confGroupRecord;

  /// Settings group title: speech engine (all ENGINE keys)
  ///
  /// In ru, this message translates to:
  /// **'движок'**
  String get confGroupEngine;

  /// Settings group title: system
  ///
  /// In ru, this message translates to:
  /// **'система'**
  String get confGroupSystem;

  /// Honest loading caption while the settings snapshot has not landed yet
  ///
  /// In ru, this message translates to:
  /// **'настройки загружаются…'**
  String get confFetching;

  /// Honest error card title when the settings snapshot fetch fails, never a daemon note
  ///
  /// In ru, this message translates to:
  /// **'не удалось получить настройки'**
  String get confFetchFailed;

  /// Label of the min_level setting
  ///
  /// In ru, this message translates to:
  /// **'порог'**
  String get confMinLevelLabel;

  /// Label of the auto_stop setting
  ///
  /// In ru, this message translates to:
  /// **'авто-стоп'**
  String get confAutoStopLabel;

  /// Label of the auto_stop_level setting
  ///
  /// In ru, this message translates to:
  /// **'порог авто-стопа'**
  String get confAutoStopLevelLabel;

  /// Label of the auto_stop_after_ms setting
  ///
  /// In ru, this message translates to:
  /// **'пауза перед стопом, мс'**
  String get confAutoStopAfterLabel;

  /// Label of the auto_stop_grace_ms setting
  ///
  /// In ru, this message translates to:
  /// **'задержка авто-стопа, мс'**
  String get confAutoStopGraceLabel;

  /// Label of the max_record_ms setting
  ///
  /// In ru, this message translates to:
  /// **'максимум записи, мс'**
  String get confMaxRecordLabel;

  /// Label of the inject_mode setting
  ///
  /// In ru, this message translates to:
  /// **'способ вставки'**
  String get confInjectModeLabel;

  /// Label of the paste_keys setting
  ///
  /// In ru, this message translates to:
  /// **'горячие клавиши вставки'**
  String get confPasteKeysLabel;

  /// Hint for the free-form paste_keys field
  ///
  /// In ru, this message translates to:
  /// **'ctrl+v'**
  String get confPasteKeysHint;

  /// Label of the clipboard_restore_ms setting
  ///
  /// In ru, this message translates to:
  /// **'возврат буфера, мс'**
  String get confClipboardRestoreLabel;

  /// Label of the atspi_allow_any_role setting
  ///
  /// In ru, this message translates to:
  /// **'atspi: любой роли'**
  String get confAtspiLabel;

  /// Label of the model setting
  ///
  /// In ru, this message translates to:
  /// **'модель'**
  String get confModelLabel;

  /// Label of the use_gpu setting
  ///
  /// In ru, this message translates to:
  /// **'gpu'**
  String get confUseGpuLabel;

  /// Label of the gpu_device setting
  ///
  /// In ru, this message translates to:
  /// **'устройство gpu'**
  String get confGpuDeviceLabel;

  /// Label of the n_threads setting
  ///
  /// In ru, this message translates to:
  /// **'потоки'**
  String get confThreadsLabel;

  /// Label of the beam_size setting
  ///
  /// In ru, this message translates to:
  /// **'размер луча'**
  String get confBeamLabel;

  /// Label of the language setting (an ENGINE key)
  ///
  /// In ru, this message translates to:
  /// **'язык'**
  String get confLanguageLabel;

  /// Label of the read-only config_path field
  ///
  /// In ru, this message translates to:
  /// **'путь конфигурации'**
  String get confConfigPathLabel;

  /// Hint marking keys that reload the engine
  ///
  /// In ru, this message translates to:
  /// **'эти настройки перезагружают модель (~секунды)'**
  String get confEngineHint;

  /// Confirm dialog title before any ENGINE set
  ///
  /// In ru, this message translates to:
  /// **'перезагрузить модель?'**
  String get confEngineConfirmTitle;

  /// Confirm dialog body explaining the engine reload
  ///
  /// In ru, this message translates to:
  /// **'изменение перезагрузит модель — на секунды диктовка будет недоступна'**
  String get confEngineConfirmBody;

  /// Confirm action in the ENGINE dialog
  ///
  /// In ru, this message translates to:
  /// **'продолжить'**
  String get confConfirm;

  /// Decline action in the ENGINE dialog
  ///
  /// In ru, this message translates to:
  /// **'отмена'**
  String get confDecline;

  /// Seconds suffix for millisecond slider labels
  ///
  /// In ru, this message translates to:
  /// **'с'**
  String get confSeconds;

  /// Milliseconds suffix for slider labels
  ///
  /// In ru, this message translates to:
  /// **'мс'**
  String get confMilliseconds;

  /// Section title for the installed models list
  ///
  /// In ru, this message translates to:
  /// **'установленные'**
  String get modelInstalledTitle;

  /// Status marker on the currently selected model
  ///
  /// In ru, this message translates to:
  /// **'активна'**
  String get modelActive;

  /// Selects an inactive model (an ENGINE set)
  ///
  /// In ru, this message translates to:
  /// **'выбрать'**
  String get modelSelectButton;

  /// Snackbar title above the daemon's verbatim model-selection refusal
  ///
  /// In ru, this message translates to:
  /// **'не удалось выбрать модель'**
  String get modelSelectFailed;

  /// Deletes an inactive model file
  ///
  /// In ru, this message translates to:
  /// **'удалить'**
  String get modelDeleteButton;

  /// Destructive confirm title for model deletion
  ///
  /// In ru, this message translates to:
  /// **'удалить модель?'**
  String get modelDeleteConfirmTitle;

  /// Destructive confirm body: deletion cannot be undone
  ///
  /// In ru, this message translates to:
  /// **'файл будет удалён безвозвратно — вернуть нельзя'**
  String get modelDeleteConfirmBody;

  /// Confirm action in the model deletion dialog
  ///
  /// In ru, this message translates to:
  /// **'удалить'**
  String get modelDeleteConfirm;

  /// Decline action in the model deletion dialog
  ///
  /// In ru, this message translates to:
  /// **'отмена'**
  String get modelDecline;

  /// Snackbar after a model file was deleted
  ///
  /// In ru, this message translates to:
  /// **'удалено'**
  String get modelDeleted;

  /// Honest empty state when settings.models is empty
  ///
  /// In ru, this message translates to:
  /// **'моделей нет — скачай из каталога ниже'**
  String get modelEmpty;

  /// Honest error row when the settings snapshot failed to load, not an empty list
  ///
  /// In ru, this message translates to:
  /// **'не удалось получить список моделей'**
  String get modelFetchFailed;

  /// Honest error line when an HF search/tree fetch throws a non-HTTP exception such as a timeout or socket error
  ///
  /// In ru, this message translates to:
  /// **'ошибка сети или таймаут'**
  String get modelFetchNetworkError;

  /// Retry the failed settings snapshot fetch from the models screen
  ///
  /// In ru, this message translates to:
  /// **'повторить'**
  String get modelFetchRetry;

  /// Honest message when the delete guard rejects a path
  ///
  /// In ru, this message translates to:
  /// **'модель активна или вне каталога'**
  String get modelDeleteBlocked;

  /// Section title for the verified model catalog
  ///
  /// In ru, this message translates to:
  /// **'каталог проверенных'**
  String get modelCatalogTitle;

  /// Catalog hint for ggml-tiny (parity with catalog.rs)
  ///
  /// In ru, this message translates to:
  /// **'сверхбыстрая, для теста'**
  String get modelNoteTiny;

  /// Catalog hint for ggml-base
  ///
  /// In ru, this message translates to:
  /// **'быстрая, простые фразы'**
  String get modelNoteBase;

  /// Catalog hint for ggml-small
  ///
  /// In ru, this message translates to:
  /// **'баланс скорости и качества'**
  String get modelNoteSmall;

  /// Catalog hint for ggml-medium
  ///
  /// In ru, this message translates to:
  /// **'хорошее качество'**
  String get modelNoteMedium;

  /// Catalog hint for ggml-large-v3-turbo
  ///
  /// In ru, this message translates to:
  /// **'лучшее качество при высокой скорости'**
  String get modelNoteLargeTurbo;

  /// Catalog hint for ggml-large-v3
  ///
  /// In ru, this message translates to:
  /// **'максимум качества, медленнее'**
  String get modelNoteLarge;

  /// Label of the UI-only VRAM limit slider
  ///
  /// In ru, this message translates to:
  /// **'лимит vram, гб'**
  String get modelVramLabel;

  /// Marker on catalog files exceeding the VRAM limit
  ///
  /// In ru, this message translates to:
  /// **'больше лимита vram'**
  String get modelVramOver;

  /// Marker on catalog files already present in settings.models
  ///
  /// In ru, this message translates to:
  /// **'уже есть'**
  String get modelAlreadyInstalled;

  /// Hint in the HF search field
  ///
  /// In ru, this message translates to:
  /// **'поиск на hugging face'**
  String get modelSearchHint;

  /// Merged semantics label of the HF search field
  ///
  /// In ru, this message translates to:
  /// **'поиск моделей на hugging face'**
  String get modelSearchSemantics;

  /// Explicit action that performs one HF fetch
  ///
  /// In ru, this message translates to:
  /// **'найти'**
  String get modelSearchButton;

  /// Pagination action: next offset, fetched only on click
  ///
  /// In ru, this message translates to:
  /// **'показать ещё'**
  String get modelShowMore;

  /// HF sort option: downloads
  ///
  /// In ru, this message translates to:
  /// **'загрузки'**
  String get modelSortDownloads;

  /// HF sort option: likes
  ///
  /// In ru, this message translates to:
  /// **'лайки'**
  String get modelSortLikes;

  /// HF sort option: lastModified
  ///
  /// In ru, this message translates to:
  /// **'обновлённые'**
  String get modelSortModified;

  /// ASR-only task filter switch
  ///
  /// In ru, this message translates to:
  /// **'только распознавание речи'**
  String get modelAsrFilter;

  /// Honest empty state for an empty HF search result
  ///
  /// In ru, this message translates to:
  /// **'ничего не нашлось'**
  String get modelResultsEmpty;

  /// Honest empty state for a repo tree without .bin files
  ///
  /// In ru, this message translates to:
  /// **'файлов .bin нет'**
  String get modelTreeEmpty;

  /// Navigate to the parent directory of a repo tree
  ///
  /// In ru, this message translates to:
  /// **'назад'**
  String get modelSubdirBack;

  /// Honest 401/404 message for HF fetch failures
  ///
  /// In ru, this message translates to:
  /// **'репозиторий закрыт или не найден'**
  String get modelRepoClosed;

  /// Label of the direct download link field
  ///
  /// In ru, this message translates to:
  /// **'прямая ссылка на .bin'**
  String get modelDirectLinkLabel;

  /// Hint for the direct download link field
  ///
  /// In ru, this message translates to:
  /// **'https://…/ggml-….bin'**
  String get modelDirectLinkHint;

  /// Primary download action
  ///
  /// In ru, this message translates to:
  /// **'скачать'**
  String get modelDownloadButton;

  /// Cancels the in-flight download
  ///
  /// In ru, this message translates to:
  /// **'отменить загрузку'**
  String get modelCancelDownload;

  /// Snackbar after a successful download (main.rs:400)
  ///
  /// In ru, this message translates to:
  /// **'модель скачана — выбери её в списке'**
  String get modelDownloadDone;

  /// Snackbar after a deliberate cancel (main.rs:374)
  ///
  /// In ru, this message translates to:
  /// **'загрузка прервана'**
  String get modelDownloadCancelled;

  /// Snackbar prefix for an honest download failure
  ///
  /// In ru, this message translates to:
  /// **'загрузка не удалась'**
  String get modelDownloadFailed;

  /// Live megabyte counter when the download size is unknown
  ///
  /// In ru, this message translates to:
  /// **'{mb} МБ'**
  String modelDownloadingMb(String mb);

  /// Tray menu item that toggles window visibility
  ///
  /// In ru, this message translates to:
  /// **'показать/скрыть окно'**
  String get trayShowHide;

  /// Tray menu item that starts recording through DictaControlNotifier
  ///
  /// In ru, this message translates to:
  /// **'начать запись'**
  String get trayStart;

  /// Tray menu item that stops recording
  ///
  /// In ru, this message translates to:
  /// **'остановить'**
  String get trayStop;

  /// Tray menu item that cancels recording
  ///
  /// In ru, this message translates to:
  /// **'отмена'**
  String get trayCancel;

  /// Tray menu item that shows the window and opens the settings tab
  ///
  /// In ru, this message translates to:
  /// **'настройки'**
  String get traySettings;

  /// Tray menu item that honestly exits the app (the only destroy call-site)
  ///
  /// In ru, this message translates to:
  /// **'выход'**
  String get trayExit;

  /// Title of the read-only hotkey card in the settings tab
  ///
  /// In ru, this message translates to:
  /// **'глобальный хоткей'**
  String get hkTitle;

  /// Label of the current evdev hotkey combination value
  ///
  /// In ru, this message translates to:
  /// **'текущая комбинация'**
  String get hkCurrentLabel;

  /// Honest value when the live daemon config has no hotkey line (default off)
  ///
  /// In ru, this message translates to:
  /// **'не задан (off)'**
  String get hkNotSet;

  /// Card error state when config.toml cannot be read
  ///
  /// In ru, this message translates to:
  /// **'не удалось прочитать конфиг демона'**
  String get hkUnavailable;

  /// Warning that editing config.toml requires a daemon restart
  ///
  /// In ru, this message translates to:
  /// **'изменение применится после рестарта демона'**
  String get hkRestartWarning;

  /// Hint that hotkey editing happens outside the app
  ///
  /// In ru, this message translates to:
  /// **'правка — только в config.toml, вне приложения'**
  String get hkEditHint;

  /// Hint about the permissions required by the daemon evdev listener
  ///
  /// In ru, this message translates to:
  /// **'evdev-путь требует группы input или udev ACL'**
  String get hkInputGroupHint;

  /// Static DE reference: KWin adapter install command (text only, never executed)
  ///
  /// In ru, this message translates to:
  /// **'KWin: Alt+V — ./systemd/install.sh && ./adapters/kwin/install.sh'**
  String get hkDeKwin;

  /// Static DE reference: Hyprland include-based hotkey wiring
  ///
  /// In ru, this message translates to:
  /// **'Hyprland: include конфигурации адаптера'**
  String get hkDeHyprland;

  /// Static DE reference: Sway include-based hotkey wiring
  ///
  /// In ru, this message translates to:
  /// **'Sway: include конфигурации адаптера'**
  String get hkDeSway;

  /// Static DE reference: GNOME apply.sh accelerator (text only, never executed)
  ///
  /// In ru, this message translates to:
  /// **'GNOME: apply.sh \'<Ctrl><Alt>v\''**
  String get hkDeGnome;

  /// Noun title of the model search section
  ///
  /// In ru, this message translates to:
  /// **'поиск моделей'**
  String get modelSearchTitle;

  /// Localized prefix for a model deletion failure snackbar
  ///
  /// In ru, this message translates to:
  /// **'не удалось удалить модель'**
  String get errDeleteFailed;

  /// Localized prefix for an own HTTP error with a status code
  ///
  /// In ru, this message translates to:
  /// **'ошибка {code}'**
  String errHttp(String code);

  /// Button that expands a long history entry to full text
  ///
  /// In ru, this message translates to:
  /// **'показать полностью'**
  String get histExpand;

  /// Button that collapses an expanded history entry back to three lines
  ///
  /// In ru, this message translates to:
  /// **'свернуть'**
  String get histCollapse;

  /// Model size in gigabytes
  ///
  /// In ru, this message translates to:
  /// **'{value} ГБ'**
  String modelSizeGb(String value);

  /// Model size in megabytes
  ///
  /// In ru, this message translates to:
  /// **'{value} МБ'**
  String modelSizeMb(String value);
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['ru'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'ru':
      return AppLocalizationsRu();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
