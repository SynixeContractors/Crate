#define COMPONENT gear
#include "\synixe\crate_client\addons\main\script_mod.hpp"
#include "\z\ace\addons\arsenal\defines.hpp"
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

#define IDC_leftRoleFilter 84001
#define IDC_rightRoleFilter 84002

#define COLOR_WHITE [1, 1, 1, 1]
#define COLOR_OWNED [0, 1, 0, 1]
#define COLOR_SALE [0, 0.75, 1, 1]
#define COLOR_INCREASE [1, 0, 0, 1]
