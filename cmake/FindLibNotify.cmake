if(NOT WIN32)
	find_package(PkgConfig REQUIRED QUIET)
	pkg_check_modules(PC_LIBNOTIFY REQUIRED QUIET libnotify)
endif()

find_path(LIBNOTIFY_INCLUDEDIR notify.h
	PATH_SUFFIXES libnotify
	HINTS ${PC_LIBNOTIFY_INCLUDE_DIRS} ${PC_LIBNOTIFY_INCLUDEDIR})

find_library(LIBNOTIFY_LIBRARY
	NAMES notify
	HINTS ${PC_LIBNOTIFY_LIBRARY_DIRS} ${PC_LIBNOTIFY_LIBDIR})

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(LibNotify DEFAULT_MSG LIBNOTIFY_INCLUDEDIR LIBNOTIFY_LIBRARY)

mark_as_advanced(LIBNOTIFY_INCLUDEDIR LIBNOTIFY_LIBRARY)

set(LIBNOTIFY_INCLUDE_DIRS ${LIBNOTIFY_INCLUDEDIR})
set(LIBNOTIFY_LIBRARIES ${LIBNOTIFY_LIBRARY})
