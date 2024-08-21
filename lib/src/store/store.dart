import 'package:get/get.dart';

abstract class Store extends GetxController {
  Future<void> ensureInitialized() async {}
}
