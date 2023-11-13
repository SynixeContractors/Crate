#define COMPONENT bodybag
#define COMPONENT_BEAUTIFIED Bodybag
#include "..\main\script_mod.hpp"

#ifdef DEBUG_ENABLED_BODYBAG
    #define DEBUG_MODE_FULL
#endif
    #ifdef DEBUG_SETTINGS_OTHER
    #define DEBUG_SETTINGS DEBUG_SETTINGS_BODYBAG
#endif

#include "..\main\script_macros.hpp"

#define DESTROY_CHANCE_DEFAULT 5
