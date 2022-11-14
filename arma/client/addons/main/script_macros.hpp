#include "\x\cba\addons\main\script_macros_common.hpp"

#define DFUNC(var1) TRIPLES(ADDON,fnc,var1)

#ifdef DISABLE_COMPILE_CACHE
    #undef PREP
    #define PREP(fncName) DFUNC(fncName) = compile preprocessFileLineNumbers QPATHTOF(functions\DOUBLES(fnc,fncName).sqf)
#else
    #undef PREP
    #define PREP(fncName) [QPATHTOF(functions\DOUBLES(fnc,fncName).sqf), QFUNC(fncName)] call CBA_fnc_compileFunction
#endif
// GUI
#define SIZEX ((safeZoneW / safeZoneH) min 1.2)
#define SIZEY (SIZEX / 1.2)
#define W_PART(num) (num * (SIZEX / 40))
#define H_PART(num) (num * (SIZEY / 25))
#define X_PART(num) (W_PART(num) + (safeZoneX + (safeZoneW - SIZEX) / 2))
#define Y_PART(num) (H_PART(num) + (safeZoneY + (safeZoneH - SIZEY) / 2))

// Loadouts
#define BLANK_LOADOUT [[],[],[],[],[],[],"","",[],["","","","","",""]]

// Extension
#define EXT "crate_server"

#define EXTCALL(function,args) private _ext_res = EXT callExtension [function, args]; \
if ((_ext_res select 1) != 0) then { ERROR_2("Error calling %1: %2",function,(_ext_res select 1)); }

#define EXTFUNC(function) private _ext_res = EXT callExtension [function, []]; \
if ((_ext_res select 1) != 0) then { ERROR_2("Error calling %1: %2",function,(_ext_res select 1)); }
