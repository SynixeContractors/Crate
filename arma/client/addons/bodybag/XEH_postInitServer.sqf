#include "script_component.hpp"

["ace_placedInBodyBag", {
    // Move all inventory from body to bodybag
    _this call FUNC(moveInventory);
}] call CBA_fnc_addEventHandler;
