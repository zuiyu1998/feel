import 'package:feel/src/helpers/helper.dart';
import 'package:shared_preferences/shared_preferences.dart';

class SharedPreferencesHelper implements Helper {
  static Future<void> ensureInitialized() async {
    final SharedPreferences prefs = await SharedPreferences.getInstance();
    getIt.registerSingleton(prefs);
  }

  static Future<void> setString(String key, String value) async {
    var prefs = getIt.get<SharedPreferences>();

    await prefs.setString(key, value);
  }

  static Future<String?> getString(String key) async {
    var prefs = getIt.get<SharedPreferences>();

    return prefs.getString(key);
  }
}
