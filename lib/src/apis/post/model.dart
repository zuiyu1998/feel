import 'package:json_annotation/json_annotation.dart';

part 'model.g.dart';

@JsonSerializable()
class Post {
  int id;
  String content;
  List<String> images;
  @JsonKey(name: "create_at")
  DateTime createAt;
  @JsonKey(name: "update_at")
  DateTime updateAt;
  int useId;

  Post(this.id, this.content, this.images, this.createAt, this.updateAt,
      this.useId);

  factory Post.fromJson(Map<String, dynamic> json) => _$PostFromJson(json);

  Map<String, dynamic> toJson() => _$PostToJson(this);
}

@JsonSerializable()
class PostSummary {
  String? content;
  List<String>? images;
  @JsonKey(name: "create_at")
  DateTime createAt;
  @JsonKey(name: "update_at")
  DateTime updateAt;

  PostSummary(
    this.content,
    this.images,
    this.createAt,
    this.updateAt,
  );

  factory PostSummary.fromJson(Map<String, dynamic> json) =>
      _$PostSummaryFromJson(json);

  Map<String, dynamic> toJson() => _$PostSummaryToJson(this);
}
