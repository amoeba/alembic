!addplugindir /usr/share/nsis/Plugins

!include LogicLib.nsh
!include "MUI2.nsh" # ModernUI

!define APP_NAME "Alembic"
!define COMP_NAME "Alembic contributors"
!define VERSION "0.1.0.0"
!define COPYRIGHT "Alembic contributors"
!define DESCRIPTION "Provides automatic updates for the voxel RPG Veloren."
!define INSTALLER_NAME "alembic-installer.exe"
!define MAIN_APP_EXE "alembic.exe"
!define ICON "../assets/alembic.ico"
!define BANNER "banner.bmp"

!define INSTALL_DIR "$PROGRAMFILES64\${APP_NAME}"
!define INSTALL_TYPE "SetShellVarContext all"
!define REG_ROOT "HKLM"
!define REG_APP_PATH "Software\Microsoft\Windows\CurrentVersion\App Paths\${MAIN_APP_EXE}"
!define UNINSTALL_PATH "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
!define REG_START_MENU "Alembic"

var SM_Folder

VIProductVersion  "${VERSION}"
VIAddVersionKey "ProductName"  "${APP_NAME}"
VIAddVersionKey "CompanyName"  "${COMP_NAME}"
VIAddVersionKey "LegalCopyright"  "${COPYRIGHT}"
VIAddVersionKey "FileDescription"  "${DESCRIPTION}"
VIAddVersionKey "FileVersion"  "${VERSION}"

SetCompressor /SOLID Lzma
Unicode true
Name "${APP_NAME}"
Caption "${APP_NAME}"
OutFile "${INSTALLER_NAME}"
BrandingText "${APP_NAME}"
InstallDirRegKey "${REG_ROOT}" "${REG_APP_PATH}" ""
InstallDir "${INSTALL_DIR}"

!define MUI_ICON "${ICON}"
!define MUI_UNICON "${ICON}"
!define MUI_WELCOMEPAGE_TEXT "Setup will guide you through the installation of ${APP_NAME}.$\r$\n\
$\r$\n\
Click Next to continue."
!define MUI_WELCOMEFINISHPAGE_BITMAP "${BANNER}"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "${BANNER}"
!define MUI_ABORTWARNING
!define MUI_UNABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\Alembic.exe"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY

!ifdef REG_START_MENU
    !define MUI_STARTMENUPAGE_DEFAULTFOLDER "${APP_NAME}"
    !define MUI_STARTMENUPAGE_REGISTRY_ROOT "${REG_ROOT}"
    !define MUI_STARTMENUPAGE_REGISTRY_KEY "${UNINSTALL_PATH}"
    !define MUI_STARTMENUPAGE_REGISTRY_VALUENAME "${REG_START_MENU}"
    !insertmacro MUI_PAGE_STARTMENU Application $SM_Folder
!endif

!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH
!insertmacro MUI_LANGUAGE "English"

Function UninstallMSI
    # $R0 should contain the GUID of the application
    StrCmp $R0 "" UninstallMSI_nomsi
    MessageBox MB_YESNOCANCEL|MB_ICONQUESTION  "A previous version of Alembic was found. It is recommended that you uninstall it first.$\n$\nDo you wish to do that now?" IDNO UninstallMSI_nomsi IDYES UninstallMSI_yesmsi
    Abort
    UninstallMSI_yesmsi:
        MessageBox MB_OK $R0
        ExecWait '"msiexec.exe" /x $R0'
    UninstallMSI_nomsi:
        DetailPrint "User declined MSI uninstallation"
FunctionEnd

Section -MainProgram
    ${INSTALL_TYPE}

    # Search for an old version installed via the previous WiX installer and remove it found
    !define upgradecode {A1365246-AB09-43F9-ACB5-07EE4DAE9EC3} ;Alembic WiX installer UpgradeCode
    System::Call 'MSI::MsiEnumRelatedProducts(t "${upgradecode}",i0,i r0,t.r1)i.r2'
    ${If} $2 = 0
        push $R0
        StrCpy $R0 $1
        Call UninstallMSI
        pop $R0
    ${EndIf}

    SetOverwrite ifnewer
    SetOutPath "$INSTDIR"

    File /r "..\..\..\target\i686-pc-windows-msvc\release\desktop.exe"
    File /r "..\..\..\target\i686-pc-windows-msvc\release\alembic.dll"
    File /r "..\..\..\target\i686-pc-windows-msvc\release\logo.png"
    Rename "$INSTDIR\desktop.exe" "$INSTDIR\Alembic.exe"
    Rename "$INSTDIR\alembic.dll" "$INSTDIR\Alembic.dll"
SectionEnd

Section -Icons_Reg
    SetOutPath "$INSTDIR"
    WriteUninstaller "$INSTDIR\uninstall.exe"

    !ifdef REG_START_MENU
        !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
        CreateDirectory "$SMPROGRAMS\$SM_Folder"
        CreateShortCut "$SMPROGRAMS\$SM_Folder\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
        CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
        CreateShortCut "$SMPROGRAMS\$SM_Folder\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe"

        !insertmacro MUI_STARTMENU_WRITE_END
    !endif

    !ifndef REG_START_MENU
        CreateDirectory "$SMPROGRAMS\${APP_NAME}"
        CreateShortCut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
        CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${MAIN_APP_EXE}"
        CreateShortCut "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\uninstall.exe"
    !endif

    WriteRegStr ${REG_ROOT} "${REG_APP_PATH}" "" "$INSTDIR\${MAIN_APP_EXE}"
    WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "DisplayName" "${APP_NAME}"
    WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "DisplayIcon" "$INSTDIR\${MAIN_APP_EXE}"
    WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "DisplayVersion" "${VERSION}"
    WriteRegStr ${REG_ROOT} "${UNINSTALL_PATH}"  "Publisher" "${COMP_NAME}"
SectionEnd

Section Uninstall
    ${INSTALL_TYPE}

    RmDir /r "$INSTDIR"

    !ifdef REG_START_MENU
        !insertmacro MUI_STARTMENU_GETFOLDER "Application" $SM_Folder
        Delete "$SMPROGRAMS\$SM_Folder\${APP_NAME}.lnk"
        Delete "$SMPROGRAMS\$SM_Folder\Uninstall ${APP_NAME}.lnk"
        Delete "$DESKTOP\${APP_NAME}.lnk"

        RmDir "$SMPROGRAMS\$SM_Folder"
    !endif

    !ifndef REG_START_MENU
        Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
        Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"

        Delete "$DESKTOP\${APP_NAME}.lnk"
        RmDir "$SMPROGRAMS\${APP_NAME}"
    !endif

    DeleteRegKey ${REG_ROOT} "${REG_APP_PATH}"
    DeleteRegKey ${REG_ROOT} "${UNINSTALL_PATH}"
SectionEnd
