package com.example.zapas.core.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.example.zapas.features.auth.presentation.ui.login.LoginScreen
import com.example.zapas.features.auth.presentation.ui.register.RegisterScreen
import com.example.zapas.features.members.presentation.ui.MemberProfileScreen
import com.example.zapas.features.profile.presentation.ui.ProfileScreen
import com.example.zapas.features.rooms.presentation.ui.CreateRoomScreen
import com.example.zapas.features.rooms.presentation.ui.RoomProfileScreen
import com.example.zapas.features.rooms.presentation.ui.RoomsScreen
import com.example.zapas.features.splash.presentation.ui.SplashScreen

@Composable
fun ZapasNavGraph(
    startDestination: Any = Splash,
    sessionViewModel: SessionViewModel = hiltViewModel()
) {
    val navController = rememberNavController()
    val sessionExpired by sessionViewModel.sessionExpired.collectAsStateWithLifecycle()
    LaunchedEffect(sessionExpired) {
        if (sessionExpired) {
            navController.navigate(Login) {
                popUpTo(0)
                launchSingleTop = true
            }
            sessionViewModel.acknowledgeSessionExpiry()
        }
    }

    NavHost(
        navController = navController, startDestination = startDestination
    ) {
        composable<Splash> {
            SplashScreen(onNavigateToLogin = {
                navController.navigate(Login) {
                    popUpTo<Splash> { inclusive = true }
                }
            }, onNavigateToMain = {
                navController.navigate(Rooms) {
                    popUpTo<Splash> { inclusive = true }
                }
            })
        }

        composable<Login> {
            LoginScreen(onLoginSuccess = {
                navController.navigate(Rooms) {
                    popUpTo<Login> { inclusive = true }
                }
            }, onNavigateToRegister = {
                navController.navigate(Register)
            })
        }

        composable<Register> {
            RegisterScreen(onRegisterComplete = {
                navController.navigate(Rooms) {
                    popUpTo<Login> { inclusive = true }
                }
            }, onBack = {
                navController.popBackStack()
            })
        }

        composable<Rooms> {
            RoomsScreen(
                onNavigateToProfile = {
                    navController.navigate(Profile)
                },
                onNavigateToCreateRoom = {
                    navController.navigate(CreateRoom)
                },
                onNavigateToRoom = { roomId ->
                    navController.navigate(RoomProfile(roomId))
                }
            )
        }

        composable<CreateRoom> {
            CreateRoomScreen(
                onBack = {
                    navController.popBackStack()
                },
                onSuccess = { roomId ->
                    navController.navigate(RoomProfile(roomId))
                }
            )
        }

        composable<RoomProfile> {
            RoomProfileScreen(
                onNavigateToMember = { memberId ->
                    navController.navigate(MemberProfile(memberId))
                },
                onBack = {
                    navController.popBackStack()
                }
            )
        }

        composable<MemberProfile> {
            MemberProfileScreen(onBack = { navController.popBackStack() })
        }

        composable<Profile> {
            ProfileScreen(onLogout = {
                navController.navigate(Login) {
                    popUpTo(0)
                }
            }, onBack = {
                navController.popBackStack()
            })
        }
    }
}
