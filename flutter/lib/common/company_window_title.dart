/// Rename only the application suffix, preserving peer and window-type labels.
String companyWindowTitle(String title, String previous, String current) {
  if (previous.isEmpty) return title;
  if (title == previous) return current;
  if (title.endsWith(' - $previous')) {
    return '${title.substring(0, title.length - previous.length)}$current';
  }
  return title;
}
