#define COMPONENT gear
#include "\synixe\crate_client\addons\main\script_mod.hpp"
#include "\z\ace\addons\arsenal\defines.hpp"

// #define DEBUG_MODE_FULL
// #define DISABLE_COMPILE_CACHE

#ifdef DEBUG_ENABLED_MAIN
    #define DEBUG_MODE_FULL
#endif
    #ifdef DEBUG_SETTINGS_MAIN
    #define DEBUG_SETTINGS DEBUG_SETTINGS_MAIN
#endif

#include "\synixe\crate_client\addons\main\script_macros.hpp"

#define IDD_RSCDISPLAYCHECKOUT 73000
#define IDC_RSCDISPLAYCHECKOUT_HEADER 73001
#define IDC_RSCDISPLAYCHECKOUT_ITEMS 73002
