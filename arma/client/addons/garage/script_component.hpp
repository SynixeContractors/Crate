#define COMPONENT garage
#include "\synixe\crate_client\addons\main\script_mod.hpp"

// #define DEBUG_MODE_FULL
// #define DISABLE_COMPILE_CACHE

#ifdef DEBUG_ENABLED_MAIN
    #define DEBUG_MODE_FULL
#endif
    #ifdef DEBUG_SETTINGS_MAIN
    #define DEBUG_SETTINGS DEBUG_SETTINGS_MAIN
#endif

#include "\synixe\crate_client\addons\main\script_macros.hpp"

#define SPAWN_TYPES [ \
    QGVAR(heli_small), GVAR(heli_medium), GVAR(heli_large), \
    QGVAR(plane_small), GVAR(plane_medium), GVAR(plane_large), \
    QGVAR(sea_small), GVAR(sea_medium), GVAR(sea_large), \
    QGVAR(land_small), GVAR(land_medium), GVAR(land_large), \
    QGVAR(thing_small), GVAR(thing_medium), GVAR(thing_large) \
]
