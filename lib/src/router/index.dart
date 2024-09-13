import 'package:feel/src/pages/home/index.dart';
import 'package:feel/src/pages/post/index.dart';
import 'package:feel/src/pages/splash/index.dart';
import 'package:feel/src/pages/sys/login/index.dart';
import 'package:get/get.dart';

class RouterBuilder {
  static const initial = Routes.splash;

  static final routes = [
    GetPage(
      name: Routes.post,
      page: () => const PostPage(),
    ),
    GetPage(
      name: Routes.splash,
      page: () => const SplashPage(),
    ),
    GetPage(
      name: Routes.home,
      page: () => const HomePage(),
    ),
    GetPage(
      name: Routes.login,
      page: () => const LoginPage(),
    )
  ];
}

abstract class Routes {
  static const splash = '/spash';
  static const home = '/home';
  static const login = '/login';
  static const post = '/post';
}
