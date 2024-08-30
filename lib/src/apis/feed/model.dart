import 'package:json_annotation/json_annotation.dart';

part 'model.g.dart';

enum FeedKind { post }

@JsonSerializable()
class FeedSource {
  int id;
  FeedKind kind;

  FeedSource(this.id, this.kind);

  factory FeedSource.fromJson(Map<String, dynamic> json) =>
      _$FeedSourceFromJson(json);

  Map<String, dynamic> toJson() => _$FeedSourceToJson(this);
}

@JsonSerializable()
class Feed {
  int id;
  FeedSource source;
  Map<String, dynamic> summary;

  Feed(this.id, this.source, this.summary);

  factory Feed.fromJson(Map<String, dynamic> json) => _$FeedFromJson(json);

  Map<String, dynamic> toJson() => _$FeedToJson(this);
}
