import 'package:feel/src/apis/post/model.dart';
import 'package:feel/src/utils/http/index.dart';

class PostApi {
  static Future<Post> getPost(int postId) async {
    var res = await defineDio.get("/api/v1/post/get_post");

    return Post.fromJson(res);
  }
}
