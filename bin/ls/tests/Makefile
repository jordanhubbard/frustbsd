ATF_TESTS_SH+=	ls_tests

# Ensure that all test cases can be run as an unprivileged user.
TEST_METADATA.ls_tests+=	required_user="unprivileged"

# Specify required files for the tests to run correctly.
TEST_METADATA.ls_tests+=	required_files="/usr/bin/awk /usr/bin/nc /usr/bin/sort"

.include <bsd.test.mk>
