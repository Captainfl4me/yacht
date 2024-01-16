using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Unity.Netcode;
using UnityEngine;

public class GameManager : MonoBehaviour
{
    private NetworkManager networkManager;
    public float table_distance = 1.5f;

    private void Awake() {
        networkManager = GetComponent<NetworkManager>();

        networkManager.OnClientStarted += NetworkManager_OnClientStarted;
        networkManager.OnClientConnectedCallback += NetworkManager_OnClientConnected;
    }

    private void NetworkManager_OnClientStarted()
    {
        if(!networkManager.IsHost)
        {
            Destroy(this);
        }
    }

    private void NetworkManager_OnClientConnected(ulong clientId)
    {
        if (!networkManager.IsHost) return;
        Debug.Log("New player!");

        float split_table_angle = 360 / networkManager.ConnectedClientsList.Count;
        GameObject[] players_list = GameObject.FindGameObjectsWithTag("Player");

        for (int i = 0; i < players_list.Length; i++)
        {
            players_list[i].transform.position = new Vector3(
                Mathf.Cos(split_table_angle * i * Mathf.Deg2Rad) * table_distance,
                0,
                Mathf.Sin(split_table_angle * i * Mathf.Deg2Rad) * table_distance
            );
            // player.rotation = Quaternion.Euler(0, split_table_angle * i, 0);
        }
        Debug.Log("New player!");
    }
}
