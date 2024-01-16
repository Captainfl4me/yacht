using Unity.Netcode;
using UnityEngine;

public class PlayerManager : NetworkBehaviour
{
    private NetworkVariable<Vector3> _networkPosition = new();
    private NetworkVariable<float> _networkRotation = new(writePerm: NetworkVariableWritePermission.Owner);

    private AudioListener audioListener;
    private Camera playerCamera;

    private void Awake()
    {
        Debug.Log("PlayerManager Awake");
        audioListener = GetComponentInChildren<AudioListener>();
        playerCamera = GetComponentInChildren<Camera>();
    }

    private void Update()
    {
        if(IsServer) {
            _networkPosition.Value = transform.position;
        } else {
            transform.position = _networkPosition.Value;
        }
        if (IsOwner) {
            _networkRotation.Value = transform.rotation.eulerAngles.y;
        }
        else {
            transform.eulerAngles = new Vector3(transform.eulerAngles.x, _networkRotation.Value, transform.eulerAngles.z);
        }
    }

    public override void OnNetworkSpawn()
    {
        if(!IsOwner) {
            audioListener.enabled = false;
            playerCamera.enabled = false;
        }
    }
}
