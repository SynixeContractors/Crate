#include "script_component.hpp"

[QGVAR(prices), {
    params ["_regular"];
    private _regular = parseNumber _regular;
    private _avgas = _regular * 1.6;
    private _jeta1 = _regular * 2.0;
    if !(player diarySubjectExists QEGVAR(discord,diary)) then {
        player createDiarySubject [
            QEGVAR(discord,diary),
            "Synixe Contractors"
        ];
    };
    GVAR(diaryRecord) = player createDiaryRecord [
        QEGVAR(discord,diary),
        [
            "Fuel Prices",
            format ["The fuel prices in this region are:<br/>Regular: $%1 per litre<br/>Avgas: $%2 per litre<br/>Jet A-1: $%3 per litre",
                _regular,
                _avgas,
                _jeta1
            ]
        ]
    ];
}] call CBA_fnc_addEventHandler;
