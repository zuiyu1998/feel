import 'package:feel/src/store/store.dart';

class UserStore extends Store {
  //token
  String? token;

  //用户信息

  bool isAuth() {
    if (token != null) {
      return true;
    } else {
      return false;
    }
  }

  String? getToken() {
    return token;
  }

  @override
  Future<void> ensureInitialized() async {}
}
