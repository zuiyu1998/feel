import 'package:feel/src/apis/user/model.dart';
import 'package:json_annotation/json_annotation.dart';

part 'model.g.dart';

enum FeedKind { post }

@JsonSerializable()
class FeedSource {
  int id;
  FeedKind kind;
  UserBaseModel user;

  FeedSource(this.id, this.kind, this.user);

  factory FeedSource.fromJson(Map<String, dynamic> json) =>
      _$FeedSourceFromJson(json);

  Map<String, dynamic> toJson() => _$FeedSourceToJson(this);
}

@JsonSerializable()
class Feed {
  int id;
  FeedSource source;
  Map<String, dynamic> summary;
  @JsonKey(name: "create_at")
  DateTime createAt;
  @JsonKey(name: "update_at")
  DateTime updateAt;

  int like;
  int share;
  int comment;

  Feed(this.id, this.source, this.summary, this.createAt, this.updateAt,
      this.like, this.comment, this.share);

  factory Feed.fromJson(Map<String, dynamic> json) => _$FeedFromJson(json);

  Map<String, dynamic> toJson() => _$FeedToJson(this);
}
