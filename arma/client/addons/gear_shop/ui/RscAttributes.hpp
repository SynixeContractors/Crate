class RscControlsGroupNoScrollbars;
class RscListNBox;
class ctrlButton;

class ace_arsenal_display {
    class controls {
        class menuBar: RscControlsGroupNoScrollbars {
            class controls {
                class buttonHide: ctrlButton {
                    onButtonClick = QUOTE([ctrlParent (_this select 0)] call FUNC(btnHide_onClick));
                };
            };
        };
        class rightTabContentListnBox: RscListNBox {
            onLBSelChanged = QUOTE(_this call FUNC(onSelChangedRightListnBox));
        };
    };
};

class ctrlControlsGroupNoScrollbars;
class RscListnBox;

class ace_arsenal_loadoutsDisplay {
    class controls {
        class centerBox: ctrlControlsGroupNoScrollbars {
            class controls {
                class contentPanel: RscListnBox {
                    columns[]={0, 0.05, 0.35, 0.45, 0.55, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90};
                };
            };
        };
    };
};
