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
