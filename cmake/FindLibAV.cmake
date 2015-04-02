# Copyright (c) 2015 Andrew Kelley
# This file is MIT licensed.
# See http://opensource.org/licenses/MIT

# LIBAV_FOUND
# LIBAV_INCLUDE_DIRS
# LIBAV_LIBRARIES

# AVFORMAT_FOUND
# AVFORMAT_INCLUDE_DIRS
# AVFORMAT_LIBRARIES

# AVCODEC_FOUND
# AVCODEC_INCLUDE_DIRS
# AVCODEC_LIBRARIES

# AVRESAMPLE_FOUND
# AVRESAMPLE_INCLUDE_DIRS
# AVRESAMPLE_LIBRARIES

# AVUTIL_FOUND
# AVUTIL_INCLUDE_DIRS
# AVUTIL_LIBRARIES

find_path(AVFORMAT_INCLUDE_DIRS NAMES libavformat/avformat.h)
find_library(AVFORMAT_LIBRARIES NAMES avformat)
if(AVFORMAT_LIBRARIES AND AVFORMAT_INCLUDE_DIRS)
  set(AVFORMAT_FOUND TRUE)
else()
  set(AVFORMAT_FOUND FALSE)
endif()

find_path(AVCODEC_INCLUDE_DIRS NAMES libavcodec/avcodec.h)
find_library(AVCODEC_LIBRARIES NAMES avcodec)
if(AVCODEC_LIBRARIES AND AVCODEC_INCLUDE_DIRS)
  set(AVCODEC_FOUND TRUE)
else()
  set(AVCODEC_FOUND FALSE)
endif()

find_path(AVRESAMPLE_INCLUDE_DIRS NAMES libavresample/avresample.h)
find_library(AVRESAMPLE_LIBRARIES NAMES avresample)
if(AVRESAMPLE_LIBRARIES AND AVRESAMPLE_INCLUDE_DIRS)
    set(AVRESAMPLE_FOUND TRUE)
else()
    set(AVRESAMPLE_FOUND FALSE)
endif()

find_path(AVUTIL_INCLUDE_DIRS NAMES libavutil/avutil.h)
find_library(AVUTIL_LIBRARIES NAMES avutil)
if(AVUTIL_LIBRARIES AND AVUTIL_INCLUDE_DIRS)
  set(AVUTIL_FOUND TRUE)
else()
  set(AVUTIL_FOUND FALSE)
endif()

if(AVFORMAT_FOUND AND AVCODEC_FOUND AND AVRESAMPLE_FOUND AND AVUTIL_FOUND)
  set(LIBAV_FOUND TRUE)
  set(LIBAV_INCLUDE_DIRS
    ${AVFORMAT_INCLUDE_DIRS}
    ${AVCODEC_INCLUDE_DIRS}
    ${AVRESAMPLE_INCLUDE_DIRS}
    ${AVUTIL_INCLUDE_DIRS})
  set(LIBAV_LIBRARIES
    ${AVFORMAT_LIBRARIES}
    ${AVCODEC_LIBRARIES}
    ${AVRESAMPLE_LIBRARIES}
    ${AVUTIL_LIBRARIES})
else()
  set(LIBAV_FOUND FALSE)
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(LIBAV DEFAULT_MSG
  AVFORMAT_LIBRARIES AVFORMAT_INCLUDE_DIRS
  AVCODEC_LIBRARIES AVCODEC_INCLUDE_DIRS
  AVRESAMPLE_LIBRARIES AVRESAMPLE_INCLUDE_DIRS
  AVUTIL_LIBRARIES AVUTIL_INCLUDE_DIRS)

mark_as_advanced(
  AVFORMAT_INCLUDE_DIRS AVFORMAT_LIBRARIES
  AVCODEC_INCLUDE_DIRS AVCODEC_LIBRARIES
  AVRESAMPLE_INCLUDE_DIRS AVRESAMPLE_LIBRARIES
  AVUTIL_INCLUDE_DIRS AVUTIL_LIBRARIES)
