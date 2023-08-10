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
    QGVAR(heli_large), QGVAR(heli_medium), QGVAR(heli_small), \
    QGVAR(plane_large), QGVAR(plane_medium), QGVAR(plane_small), \
    QGVAR(sea_large), QGVAR(sea_medium), QGVAR(sea_small), \
    QGVAR(land_large), QGVAR(land_medium), QGVAR(land_small), \
    QGVAR(thing_large), QGVAR(thing_medium), QGVAR(thing_small), \
]
