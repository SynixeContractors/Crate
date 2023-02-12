#include "script_component.hpp"

[
    [
        [getMissionConfigValue ["onLoadName", ""], "<t size = '1.5' underline = '1'>%1</t><br/>"],
        ["Synixe Contractors"],
        [getText (configFile >> "CfgWorlds" >> worldName >> "description"), "<t size = '1' font='puristaSemiBold'>%1</t>", 70]
    ], 0.75, 0.75, "<t align = 'center' shadow = '1' size = '1.0'>%1</t>"
] spawn BIS_fnc_typeText;
