import 'package:feel/src/apis/common/model.dart';
import 'package:feel/src/apis/feed/model.dart';
import 'package:feel/src/utils/http/index.dart';

class FeedApi {
  static Future<PageList<Feed>> getFeedPagelist() async {
    var res = await defineDio.get("/api/v1/feed/get_feed_page_list");
    return PageList<Feed>.fromJson(res, (json) {
      return Feed.fromJson(json);
    });
  }
}
