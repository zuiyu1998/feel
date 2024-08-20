import 'package:get/get.dart';

class SplashController extends GetxController {
  var isNeedUpdate = false;
  var isAuth = false;

  void setNeedUpdate(bool v) {
    isNeedUpdate = v;
  }

  void setAuth(bool v) {
    isAuth = v;
  }
}
