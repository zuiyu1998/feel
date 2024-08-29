import 'package:feel/src/helpers/shared_preferences_helper.dart';

export 'helper.dart';

Future<void> initHelpers() async {
  await SharedPreferencesHelper.ensureInitialized();
}
