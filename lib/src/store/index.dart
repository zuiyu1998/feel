import 'package:feel/src/store/modules/user.dart';
import 'package:feel/src/store/store.dart';
import 'package:get/get.dart';

Future<void> initialized<T extends Store>(T store) async {
  await store.ensureInitialized();
  Get.put(store, permanent: true);
}

Future<void> initStore() async {
  var stores = [UserStore()];

  for (var store in stores) {
    await initialized(store);
  }
}
