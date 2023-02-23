#define COMPONENT log
#include "..\main\script_mod.hpp"

#ifdef DEBUG_ENABLED_LOG
    #define DEBUG_MODE_FULL
#endif
    #ifdef DEBUG_SETTINGS_LOG
    #define DEBUG_SETTINGS DEBUG_SETTINGS_LOG
#endif

#include "..\main\script_macros.hpp"
