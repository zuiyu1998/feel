import 'package:feel/src/store/store.dart';

class UserStore extends Store {
  //token
  String? token;

  //用户信息

  isAuth() {
    if (token != null) {
      return true;
    } else {
      return false;
    }
  }

  getToken() {
    return token;
  }

  @override
  Future<void> ensureInitialized() async {}
}
