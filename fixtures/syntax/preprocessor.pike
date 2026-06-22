// Pike 8.0.1116 preprocessor fixture.
// Directive set is drawn from pikelang/Pike refdoc/preprocessor.xml
// <section title="Preprocessor Directives">.

#ifndef PIKE_ZED_FIXTURE_H
#define PIKE_ZED_FIXTURE_H
#include <stdio.h>
#include "local.pike"

#if PIKE_ZED_FIXTURE_H
#warning fixture only: PIKE_ZED_FIXTURE_H is defined
#endif

#ifdef __PIKE__
#define PIKE_VERSION_AT_LEAST(maj, min) \
  ((__REAL_MAJOR__ > (maj)) || \
   ((__REAL_MAJOR__ == (maj)) && (__REAL_MINOR__ >= (min))))
#else
#error "this fixture requires a Pike compiler"
#endif

#if PIKE_VERSION_AT_LEAST(8, 0)
constant supports_pike_8 = 1;
#endif

#pragma Pike.compilation_level 1

#endif
