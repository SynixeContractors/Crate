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
    QGVAR(heli_small), QGVAR(heli_medium), QGVAR(heli_large), \
    QGVAR(plane_small), QGVAR(plane_medium), QGVAR(plane_large), \
    QGVAR(sea_small), QGVAR(sea_medium), QGVAR(sea_large), \
    QGVAR(land_small), QGVAR(land_medium), QGVAR(land_large), \
    QGVAR(thing_small), QGVAR(thing_medium), QGVAR(thing_large) \
]
