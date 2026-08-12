import 'package:web/web.dart' as web;

void changeFavicon(String url) {
  try {
    final link = web.document.querySelector("link[rel*='icon']") as web.HTMLLinkElement?;
    if (link != null) {
      link.href = url;
    } else {
      final newLink = web.document.createElement('link') as web.HTMLLinkElement;
      newLink.rel = 'icon';
      newLink.href = url;
      web.document.head?.appendChild(newLink);
    }
  } catch (e) {
    // ignore
  }
}

void openUrl(String url) {
  try {
    web.window.open(url, '_blank');
  } catch (e) {
    // ignore
  }
}

void saveLocalData(String key, String value) {
  try {
    web.window.localStorage.setItem(key, value);
  } catch (_) {}
}

String? getLocalData(String key) {
  try {
    return web.window.localStorage.getItem(key);
  } catch (_) {
    return null;
  }
}


