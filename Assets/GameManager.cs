using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Unity.Netcode;
using UnityEngine;

public class GameManager : NetworkBehaviour
{
    public NetworkManager networkManager;
    public float tableDistance = 1.5f;
    public float spawnHeight = 1f;

    private bool gameStarted = false;

    private void Awake() {
        networkManager.OnClientConnectedCallback += NetworkManager_OnClientConnected;
    }

    private void NetworkManager_OnClientConnected(ulong clientId)
    {
        if (!networkManager.IsHost) return;

        if (gameStarted) {
            Debug.Log("Game already started!");
            networkManager.DisconnectClient(clientId);
            return;
        }

        float split_table_angle = 360 / networkManager.ConnectedClientsList.Count;
        GameObject[] players_list = GameObject.FindGameObjectsWithTag("Player");

        for (int i = 0; i < players_list.Length; i++)
        {
            players_list[i].transform.SetPositionAndRotation(new Vector3(
                Mathf.Cos(split_table_angle * i * Mathf.Deg2Rad) * tableDistance,
                spawnHeight,
                Mathf.Sin(split_table_angle * i * Mathf.Deg2Rad) * tableDistance
            ), Quaternion.Euler(0, -90.0f + split_table_angle * i, 0));
        }
        Debug.Log("New player!");
    }

    private void Start() {
        if(networkManager.IsHost) {
            gameStarted = true;
            NextRound();
        }
    }

    private void NextRound() {
        networkManager.StartClient();
    }
}
