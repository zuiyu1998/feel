import 'package:feel/src/store/modules/user.dart';
import 'package:feel/src/store/store.dart';
import 'package:get/get.dart';

Future<void> initialized<T extends Store>(T store) async {
  await store.ensureInitialized();

  Get.replace<T>(
    store,
  );
}

Future<void> initStore() async {
  var stores = [UserStore()];

  for (var store in stores) {
    await initialized(store);
  }
}
