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

  /// Weak secondary text on placeholder screens
  ///
  /// In ru, this message translates to:
  /// **'раздел появится в следующих фазах'**
  String get placeholderScreen;

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

  /// Live daemon state label on the dashboard
  ///
  /// In ru, this message translates to:
  /// **'состояние: {state}'**
  String stateLabel(String state);

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
  /// **'модель: данных нет'**
  String get diagModelUnavailable;

  /// Neutral row while the settings snapshot has not landed yet
  ///
  /// In ru, this message translates to:
  /// **'данные модели загружаются…'**
  String get diagModelFetching;

  /// Distinct row when the settings fetch or parse failed
  ///
  /// In ru, this message translates to:
  /// **'модель: не удалось получить данные'**
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
