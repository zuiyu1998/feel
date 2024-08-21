import 'package:feel/src/pages/splash/index.dart';
import 'package:get/get.dart';

class RouterBuilder {
  static const initial = Routes.splash;

  static final routes = [
    GetPage(
      name: Routes.splash,
      page: () => const SplashPage(),
    )
  ];
}

abstract class Routes {
  static const splash = '/spash';
}
