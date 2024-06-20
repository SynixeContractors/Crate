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

class RscListNBox;

#include "AceArsenalSorts.hpp"
#include "Cfg3DEN.hpp"
#include "CfgEventHandlers.hpp"
#include "CfgVehicles.hpp"
#include "ui\ACE_Arsenal.hpp"
#include "ui\RscCheckout.hpp"
