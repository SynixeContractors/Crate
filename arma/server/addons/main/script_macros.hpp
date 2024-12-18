#include "\x\cba\addons\main\script_macros_common.hpp"

#define DFUNC(var1) TRIPLES(ADDON,fnc,var1)

#ifdef DISABLE_COMPILE_CACHE
    #undef PREP
    #define PREP(fncName) DFUNC(fncName) = compile preprocessFileLineNumbers QPATHTOF(functions\DOUBLES(fnc,fncName).sqf)
#else
    #undef PREP
    #define PREP(fncName) [QPATHTOF(functions\DOUBLES(fnc,fncName).sqf), QFUNC(fncName)] call CBA_fnc_compileFunction
#endif

#define EXT "crate_server"

#define EXTCALL(function,args) private _ext_res = EXT callExtension [function, args]; \
if ((_ext_res select 1) != 0) then { \
    ERROR_2("Error calling %1: %2",function,(_ext_res select 1)); \
    ERROR_1("ARGS: %1",args); \
}

#define EXTFUNC(function) EXTCALL(function,[])
