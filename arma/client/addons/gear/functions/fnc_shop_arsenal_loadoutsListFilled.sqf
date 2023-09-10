#include "script_component.hpp"

params ["_display", "_control"];

if !(GVAR(enabled)) exitWith {};

private _contentPanelCtrl = _display displayCtrl IDC_contentPanel;

(lnbSize _contentPanelCtrl) params ["_rows", "_columns"];

if (ace_arsenal_currentLoadoutsTab != IDC_buttonSharedLoadouts) then {
    private _sharingEnabled = (ace_arsenal_allowSharedLoadouts && {isMultiplayer});
    if (ace_arsenal_currentLoadoutsTab == IDC_buttonDefaultLoadouts || {!_sharingEnabled}) then {
        _contentPanelCtrl lnbSetColumnsPos [0, 0, 0.35, 0.45, 0.55, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90];
    } else {
        _contentPanelCtrl lnbSetColumnsPos [0, 0.05, 0.35, 0.45, 0.55, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90];
    };
} else {
    _contentPanelCtrl lnbSetColumnsPos [0, 0.15, 0.35, 0.45, 0.55, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90];
};

for "_lbIndex" from 0 to (_rows - 1) do {
    private _loadoutName = _contentPanelCtrl lnbText [_lbIndex, 1];
    private _loadout = switch ace_arsenal_currentLoadoutsTab do {
        case IDC_buttonMyLoadouts;
        case IDC_buttonDefaultLoadouts: {
            (_contentPanelCtrl getVariable _loadoutName + str ace_arsenal_currentLoadoutsTab) select 0
        };
        case IDC_buttonSharedLoadouts:{
            (ace_arsenal_sharedLoadoutsNamespace getVariable ((_contentPanelCtrl lnbText [_lbIndex, 0]) + (_contentPanelCtrl lnbText [_lbIndex, 1]))) select 2
        };
    };

    if (count _loadout != 2) then {
        _loadout = [_loadout, []];
    };

    private _cost = [[_loadout] call FUNC(loadout_items), 0, false] call FUNC(shop_items_cost);
    _contentPanelCtrl lnbSetText [[_lbIndex, 10], format ["$%1", _cost]];
};
