import 'download_helper_stub.dart'
    if (dart.library.html) 'download_helper_web.dart' as impl;

void downloadFileFromUrl(String url, {String? filename}) {
  impl.downloadFileFromUrl(url, filename: filename);
}
