# Define the package category
PACKAGE=runtime

# List of scripts to be installed
SCRIPTS = freebsd-version

# Man pages to be installed
MAN = freebsd-version.1

# Files to clean up after build
CLEANFILES = freebsd-version freebsd-version.sh

# Path to the newvers.sh script
NEWVERS = ${SRCTOP}/sys/conf/newvers.sh

# Generate freebsd-version.sh from freebsd-version.sh.in using variables from newvers.sh
freebsd-version.sh: ${.CURDIR}/freebsd-version.sh.in ${NEWVERS}
	eval $$(sh ${NEWVERS} -v); \
	if ! sed -e "\
			s/@@TYPE@@/$${TYPE}/g; \
			s/@@REVISION@@/$${REVISION}/g; \
			s/@@BRANCH@@/$${BRANCH}/g; \
		" ${.CURDIR}/freebsd-version.sh.in >${.TARGET} ; then \
		rm -f ${.TARGET} ; \
		exit 1 ; \
	fi

# Include the FreeBSD program build rules
.include <bsd.prog.mk>
