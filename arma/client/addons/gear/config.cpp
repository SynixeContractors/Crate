#include "script_component.hpp"

class CfgPatches {
    class ADDON {
        name = QUOTE(COMPONENT);
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {"crate_client_main", "ace_arsenal"};
        author = "AUTHOR";
        VERSION_CONFIG;
    };
};

#include "Cfg3DEN.hpp"
#include "CfgEventHandlers.hpp"
#include "ui\ACE_Arsenal.hpp"
#include "ui\RscCheckout.hpp"
#include "AceArsenalSorts.hpp"
