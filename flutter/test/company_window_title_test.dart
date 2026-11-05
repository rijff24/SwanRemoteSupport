import 'package:flutter_hbb/common/company_window_title.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('live company rename preserves peer and window type', () {
    expect(companyWindowTitle('Company A', 'Company A', 'Company B'), 'Company B');
    expect(
        companyWindowTitle('Customer 7 - Remote Desktop - Company A',
            'Company A', 'Company B'),
        'Customer 7 - Remote Desktop - Company B');
    expect(
        companyWindowTitle('Customer 7 - File Transfer - Company A',
            'Company A', 'Company B'),
        'Customer 7 - File Transfer - Company B');
    expect(companyWindowTitle('Company A - Customer 7', 'Company A', 'Company B'),
        'Company A - Customer 7');
    expect(companyWindowTitle('Peer - ', '', 'Company B'), 'Peer - ');
    expect(companyWindowTitle('Peer - 株式会社', '株式会社', 'Support 🦢'),
        'Peer - Support 🦢');
  });
}
