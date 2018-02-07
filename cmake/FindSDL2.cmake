if(NOT CMAKE_CROSSCOMPILING)
  find_package(PkgConfig QUIET)
  pkg_check_modules(PC_SDL2 sdl2)
endif()

set_extra_dirs_lib(SDL2 sdl)
find_library(SDL2_LIBRARY
  NAMES SDL2
  HINTS ${HINTS_SDL2_LIBDIR} ${PC_SDL2_LIBDIR} ${PC_SDL2_LIBRARY_DIRS}
  PATHS ${PATHS_SDL2_LIBDIR}
  ${CROSSCOMPILING_NO_CMAKE_SYSTEM_PATH}
)
set(CMAKE_FIND_FRAMEWORK FIRST)
set_extra_dirs_include(SDL2 sdl "${SDL2_LIBRARY}")
# Looking for 'SDL.h' directly might accidently find a SDL instead of SDL 2
# installation. Look for a header file only present in SDL 2 instead.
find_path(SDL2_INCLUDEDIR SDL_assert.h
  PATH_SUFFIXES SDL2
  HINTS ${HINTS_SDL2_INCLUDEDIR} ${PC_SDL2_INCLUDEDIR} ${PC_SDL2_INCLUDE_DIRS}
  PATHS ${PATHS_SDL2_INCLUDEDIR}
)

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(SDL2 DEFAULT_MSG SDL2_LIBRARY SDL2_INCLUDEDIR)

mark_as_advanced(SDL2_LIBRARY SDL2_INCLUDEDIR)

if(SDL2_FOUND)
  set(SDL2_LIBRARIES ${SDL2_LIBRARY})
  set(SDL2_INCLUDE_DIRS ${SDL2_INCLUDEDIR})

  add_library(Deps::SDL2 UNKNOWN IMPORTED)
  set_target_properties(Deps::SDL2 PROPERTIES
    IMPORTED_LOCATION "${SDL2_LIBRARY}"
    INTERFACE_INCLUDE_DIRECTORIES "${SDL2_INCLUDEDIR}"
  )

  is_bundled(SDL2_BUNDLED "${SDL2_LIBRARY}")
  if(SDL2_BUNDLED AND TARGET_OS STREQUAL "windows")
    set(SDL2_COPY_FILES "${EXTRA_SDL2_LIBDIR}/SDL2.dll")
  else()
    set(SDL2_COPY_FILES)
  endif()
endif()
