#include "base/rust.h"
#include "engine/console.h"
#include "engine/rust.h"
#include <cstdint>
#include <type_traits>
#include <utility>

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

#ifndef CXXBRIDGE1_RELOCATABLE
#define CXXBRIDGE1_RELOCATABLE
namespace detail {
template <typename... Ts>
struct make_void {
  using type = void;
};

template <typename... Ts>
using void_t = typename make_void<Ts...>::type;

template <typename Void, template <typename...> class, typename...>
struct detect : std::false_type {};
template <template <typename...> class T, typename... A>
struct detect<void_t<T<A...>>, T, A...> : std::true_type {};

template <template <typename...> class T, typename... A>
using is_detected = detect<void, T, A...>;

template <typename T>
using detect_IsRelocatable = typename T::IsRelocatable;

template <typename T>
struct get_IsRelocatable
    : std::is_same<typename T::IsRelocatable, std::true_type> {};
} // namespace detail

template <typename T>
struct IsRelocatable
    : std::conditional<
          detail::is_detected<detail::detect_IsRelocatable, T>::value,
          detail::get_IsRelocatable<T>,
          std::integral_constant<
              bool, std::is_trivially_move_constructible<T>::value &&
                        std::is_trivially_destructible<T>::value>>::type {};
#endif // CXXBRIDGE1_RELOCATABLE
} // namespace cxxbridge1
} // namespace rust

using IConsole_IResult = ::IConsole_IResult;
using IConsole = ::IConsole;

static_assert(
    ::rust::IsRelocatable<::ColorRGBA>::value,
    "type ColorRGBA should be trivially move constructible and trivially destructible in C++ to be used as an argument of `Print` in Rust");
static_assert(
    ::rust::IsRelocatable<::StrRef>::value,
    "type StrRef should be trivially move constructible and trivially destructible in C++ to be used as an argument of `Register`, `Print` in Rust");
static_assert(
    ::rust::IsRelocatable<::UserPtr>::value,
    "type UserPtr should be trivially move constructible and trivially destructible in C++ to be used as an argument of `Register` in Rust");
static_assert(
    ::rust::IsRelocatable<::IConsole_FCommandCallback>::value,
    "type IConsole_FCommandCallback should be trivially move constructible and trivially destructible in C++ to be used as an argument of `Register` in Rust");

extern "C" {
void cxxbridge1$IConsole$Register(::IConsole &self, ::StrRef *pName, ::StrRef *pParams, ::std::int32_t Flags, ::IConsole_FCommandCallback *pfnFunc, ::UserPtr *pUser, ::StrRef *pHelp) noexcept {
  void (::IConsole::*Register$)(::StrRef, ::StrRef, ::std::int32_t, ::IConsole_FCommandCallback, ::UserPtr, ::StrRef) = &::IConsole::Register;
  (self.*Register$)(::std::move(*pName), ::std::move(*pParams), Flags, ::std::move(*pfnFunc), ::std::move(*pUser), ::std::move(*pHelp));
}

void cxxbridge1$IConsole$Print(const ::IConsole &self, ::std::int32_t Level, ::StrRef *pFrom, ::StrRef *pStr, ::ColorRGBA *PrintColor) noexcept {
  void (::IConsole::*Print$)(::std::int32_t, ::StrRef, ::StrRef, ::ColorRGBA) const = &::IConsole::Print;
  (self.*Print$)(Level, ::std::move(*pFrom), ::std::move(*pStr), ::std::move(*PrintColor));
}
} // extern "C"
